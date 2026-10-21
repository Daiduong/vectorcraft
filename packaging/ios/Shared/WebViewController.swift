import UIKit
import UniformTypeIdentifiers
import WebKit

/// Full-screen WKWebView running a Craft app's web build (apps/<app>-web) from the in-app server.
///
/// Native glue the browser can't do on its own:
/// - Save / Export: the page's blob downloads become `WKDownload`s, then a "Save to Files" sheet;
/// - files opened from the Files app (or shared to the app) go to `window.<CraftOpenFileAPI>.openFile`
///   when the page has it (VectorCraft), or else arrive as a file dropped on the canvas.
final class WebViewController: UIViewController {
    private var webView: WKWebView!
    private var server: LocalServer?
    private var pendingFiles: [URL] = []
    private var pageLoaded = false
    /// A delivery loop is working through `pendingFiles`; only one runs at a time, so a file isn't
    /// opened twice or dropped from the queue undelivered.
    private var delivering = false
    /// Where each in-flight download is being written.
    private var downloadTargets: [ObjectIdentifier: URL] = [:]
    private let appName = Bundle.main.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String ?? "App"
    private let openFileAPI = Bundle.main.object(forInfoDictionaryKey: "CraftOpenFileAPI") as? String ?? ""

    override var prefersStatusBarHidden: Bool { true }
    override var prefersHomeIndicatorAutoHidden: Bool { true }
    // Let the page see edge swipes (the canvas pans and zooms with them).
    override var preferredScreenEdgesDeferringSystemGestures: UIRectEdge { .all }

    override func loadView() {
        let config = WKWebViewConfiguration()
        config.websiteDataStore = .default()
        config.preferences.isElementFullscreenEnabled = true
        config.defaultWebpagePreferences.allowsContentJavaScript = true
        config.allowsInlineMediaPlayback = true
        config.mediaTypesRequiringUserActionForPlayback = []
        // Touch rules VectorCraft's page has and the other apps' pages don't yet: the canvas takes
        // every touch (`touch-action: none`, so WebKit doesn't keep fingers for scrolling or zooming),
        // no overscroll, text selection, callouts or tap highlight.
        let css = """
        html, body { overscroll-behavior: none; -webkit-user-select: none; user-select: none;
          -webkit-touch-callout: none; -webkit-tap-highlight-color: transparent; -webkit-text-size-adjust: 100%; }
        canvas { touch-action: none; }
        """
        let inject = "(() => { const s = document.createElement('style'); s.textContent = \(Self.jsString(css)); document.head.appendChild(s); })()"
        config.userContentController.addUserScript(
            WKUserScript(source: inject, injectionTime: .atDocumentEnd, forMainFrameOnly: true))
        #if DEBUG
        // The page's console and uncaught errors in the app's log (`simctl launch --console`).
        let bridge = """
        (() => {
          const post = (level, args) => { try { webkit.messageHandlers.console.postMessage(level + ": " +
            Array.from(args).map(a => a instanceof Error ? a.stack || String(a) : typeof a === "object" ? JSON.stringify(a) : String(a)).join(" ")); } catch (_) {} };
          for (const level of ["log", "info", "warn", "error", "debug"]) {
            const orig = console[level].bind(console);
            console[level] = (...args) => { post(level, args); orig(...args); };
          }
          addEventListener("error", e => post("uncaught", [e.message, e.filename + ":" + e.lineno]));
          addEventListener("unhandledrejection", e => post("unhandled rejection", [e.reason]));
        })()
        """
        config.userContentController.addUserScript(
            WKUserScript(source: bridge, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        config.userContentController.add(ConsoleLogger(), name: "console")
        #endif

        webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.isOpaque = false
        webView.backgroundColor = UIColor(red: 0x26 / 255, green: 0x26 / 255, blue: 0x26 / 255, alpha: 1)
        webView.scrollView.isScrollEnabled = false
        webView.scrollView.bounces = false
        webView.scrollView.contentInsetAdjustmentBehavior = .never
        webView.scrollView.pinchGestureRecognizer?.isEnabled = false
        webView.allowsBackForwardNavigationGestures = false
        webView.allowsLinkPreview = false
        #if DEBUG
        webView.isInspectable = true // Safari › Develop › <iPad> to debug the page
        #endif
        view = webView
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        guard let site = Bundle.main.url(forResource: "web", withExtension: nil),
              FileManager.default.fileExists(atPath: site.appendingPathComponent("index.html").path) else {
            showError("The web build is missing from the app bundle. Run packaging/ios/build.sh <app>, then build again.")
            return
        }
        let server = LocalServer(root: site)
        self.server = server
        server.start { [weak self] port in
            guard let self else { return }
            guard let port, let url = URL(string: "http://localhost:\(port)/index.html") else {
                self.showError("Couldn't start the local web server.")
                return
            }
            self.webView.load(URLRequest(url: url))
        }
    }

    // MARK: - Opening files

    /// Queue `urls` for the page; they open once the app inside it is running.
    func open(_ urls: [URL]) {
        pendingFiles.append(contentsOf: urls)
        if pageLoaded { deliverPendingFiles() }
    }

    /// Starts delivering the queued files, unless a delivery loop is already running (it picks up
    /// files queued since).
    private func deliverPendingFiles() {
        guard !delivering else { return }
        deliverNext(attempt: 0)
    }

    private func deliverNext(attempt: Int) {
        guard let url = pendingFiles.first else {
            delivering = false
            return
        }
        delivering = true
        let scoped = url.startAccessingSecurityScopedResource()
        let data = try? Data(contentsOf: url)
        if scoped { url.stopAccessingSecurityScopedResource() }
        guard let data else {
            pendingFiles.removeFirst()
            showError("Couldn't read \(url.lastPathComponent).")
            deliverNext(attempt: 0)
            return
        }
        let args = (try? JSONSerialization.data(withJSONObject: [url.lastPathComponent, openFileAPI]))
            .flatMap { String(data: $0, encoding: .utf8) } ?? "[\"file\", \"\"]"
        // Returns false until the app inside the page has started (its loading text is still up, or
        // `openFile` refuses the file): retry.
        let js = """
        (() => {
          const [name, api] = \(args);
          const vc = api ? window[api] : null;
          const canvas = document.querySelector("canvas");
          const loading = document.querySelector("[id$='_loading']");
          if (!canvas || (vc ? !vc.openFile : loading)) return false;
          const bin = atob("\(data.base64EncodedString())");
          const bytes = new Uint8Array(bin.length);
          for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
          if (vc) return vc.openFile(name, bytes) === true;
          // No API: hand the file over the way a drag and drop onto the canvas does.
          const dt = new DataTransfer();
          dt.items.add(new File([bytes], name));
          const r = canvas.getBoundingClientRect();
          const at = { clientX: r.left + r.width / 2, clientY: r.top + r.height / 2, bubbles: true, cancelable: true, dataTransfer: dt };
          canvas.dispatchEvent(new DragEvent("dragenter", at));
          canvas.dispatchEvent(new DragEvent("dragover", at));
          canvas.dispatchEvent(new DragEvent("drop", at));
          return true;
        })()
        """
        webView.evaluateJavaScript(js) { [weak self] result, _ in
            guard let self else { return }
            if (result as? Bool) == true {
                self.pendingFiles.removeFirst()
                self.deliverNext(attempt: 0)
            } else if attempt < 120 {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.deliverNext(attempt: attempt + 1) }
            } else {
                // Give up on this file; the rest wait for the next open or page load.
                self.pendingFiles.removeFirst()
                self.delivering = false
                self.showError("\(self.appName) didn't start in time to open \(url.lastPathComponent).")
            }
        }
    }

    // MARK: - Helpers

    /// `s` as a JavaScript string literal.
    private static func jsString(_ s: String) -> String {
        (try? JSONSerialization.data(withJSONObject: [s])).flatMap { String(data: $0, encoding: .utf8) }
            .map { String($0.dropFirst().dropLast()) } ?? "\"\""
    }

    private func showError(_ message: String) {
        let alert = UIAlertController(title: appName, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }

    private func exportToFiles(_ file: URL) {
        let picker = UIDocumentPickerViewController(forExporting: [file], asCopy: true)
        picker.shouldShowFileExtensions = true
        present(picker, animated: true)
    }
}

// MARK: - Navigation and downloads

extension WebViewController: WKNavigationDelegate {
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        pageLoaded = true
        deliverPendingFiles()
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        if navigationAction.shouldPerformDownload || navigationAction.request.url?.scheme == "blob" {
            decisionHandler(.download)
            return
        }
        // Links off the app (help pages, licences…) open in Safari.
        if let url = navigationAction.request.url, url.host != "localhost", url.scheme == "http" || url.scheme == "https",
           navigationAction.targetFrame?.isMainFrame ?? true {
            UIApplication.shared.open(url)
            decisionHandler(.cancel)
            return
        }
        decisionHandler(.allow)
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationResponse: WKNavigationResponse,
                 decisionHandler: @escaping (WKNavigationResponsePolicy) -> Void) {
        decisionHandler(navigationResponse.canShowMIMEType ? .allow : .download)
    }

    func webView(_ webView: WKWebView, navigationAction: WKNavigationAction, didBecome download: WKDownload) {
        download.delegate = self
    }

    func webView(_ webView: WKWebView, navigationResponse: WKNavigationResponse, didBecome download: WKDownload) {
        download.delegate = self
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        // Memory pressure killed the page: reload it (Data Recovery offers the unsaved work).
        webView.reload()
    }
}

extension WebViewController: WKDownloadDelegate {
    func download(_ download: WKDownload, decideDestinationUsing response: URLResponse, suggestedFilename: String,
                  completionHandler: @escaping (URL?) -> Void) {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        } catch {
            completionHandler(nil)
            return
        }
        let target = dir.appendingPathComponent(suggestedFilename.isEmpty ? "Untitled" : suggestedFilename)
        downloadTargets[ObjectIdentifier(download)] = target
        completionHandler(target)
    }

    func downloadDidFinish(_ download: WKDownload) {
        guard let file = downloadTargets.removeValue(forKey: ObjectIdentifier(download)) else { return }
        exportToFiles(file)
    }

    func download(_ download: WKDownload, didFailWithError error: Error, resumeData: Data?) {
        downloadTargets.removeValue(forKey: ObjectIdentifier(download))
        showError("Save failed: \(error.localizedDescription)")
    }
}

// MARK: - JavaScript dialogs and new windows

extension WebViewController: WKUIDelegate {
    func webView(_ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration,
                 for navigationAction: WKNavigationAction, windowFeatures: WKWindowFeatures) -> WKWebView? {
        if let url = navigationAction.request.url {
            if url.scheme == "blob" {
                webView.load(navigationAction.request) // becomes a download
            } else {
                UIApplication.shared.open(url)
            }
        }
        return nil
    }

    func webView(_ webView: WKWebView, runJavaScriptAlertPanelWithMessage message: String,
                 initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping () -> Void) {
        let alert = UIAlertController(title: nil, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in completionHandler() })
        present(alert, animated: true)
    }

    func webView(_ webView: WKWebView, runJavaScriptConfirmPanelWithMessage message: String,
                 initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping (Bool) -> Void) {
        let alert = UIAlertController(title: nil, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { _ in completionHandler(false) })
        alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in completionHandler(true) })
        present(alert, animated: true)
    }
}

#if DEBUG
private final class ConsoleLogger: NSObject, WKScriptMessageHandler {
    func userContentController(_ controller: WKUserContentController, didReceive message: WKScriptMessage) {
        print("[page] \(message.body)")
    }
}
#endif
