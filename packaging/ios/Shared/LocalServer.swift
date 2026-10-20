import Foundation
import Network

/// A tiny static-file HTTP server for the bundled web build, bound to the loopback interface.
///
/// `http://localhost` is a secure context, so WebGPU and the async clipboard work, and `.wasm` is
/// served as `application/wasm` so it stream-compiles. A fixed port keeps the page's origin (and so
/// its IndexedDB recovery data and preferences) the same from launch to launch.
final class LocalServer {
    /// `CraftServerPort` from Info.plist: each app has its own.
    static let preferredPort: UInt16 = (Bundle.main.object(forInfoDictionaryKey: "CraftServerPort") as? NSNumber)?.uint16Value ?? 47_861

    private let root: URL
    private var listener: NWListener?
    private let queue = DispatchQueue(label: "craft.localserver", attributes: .concurrent)
    private(set) var port: UInt16 = 0

    init(root: URL) {
        self.root = root.standardizedFileURL
    }

    /// Start listening; calls `ready` on the main queue with the port, or nil on failure.
    func start(ready: @escaping (UInt16?) -> Void) {
        start(on: Self.preferredPort, fallback: true, ready: ready)
    }

    private func start(on port: UInt16, fallback: Bool, ready: @escaping (UInt16?) -> Void) {
        let params = NWParameters.tcp
        params.allowLocalEndpointReuse = true
        params.requiredInterfaceType = .loopback
        guard let nwPort = NWEndpoint.Port(rawValue: port),
              let listener = try? NWListener(using: params, on: nwPort) else {
            if fallback { start(on: 0, fallback: false, ready: ready) } else { DispatchQueue.main.async { ready(nil) } }
            return
        }
        self.listener = listener
        var reported = false
        listener.stateUpdateHandler = { [weak self] state in
            guard let self else { return }
            switch state {
            case .ready:
                guard !reported else { return }
                reported = true
                self.port = listener.port?.rawValue ?? port
                DispatchQueue.main.async { ready(self.port) }
            case .failed:
                listener.cancel()
                guard !reported else { return }
                reported = true
                // The preferred port is taken: any free one still works (the origin just differs).
                if fallback { self.start(on: 0, fallback: false, ready: ready) } else { DispatchQueue.main.async { ready(nil) } }
            default:
                break
            }
        }
        listener.newConnectionHandler = { [weak self] connection in
            self?.handle(connection)
        }
        listener.start(queue: queue)
    }

    func stop() {
        listener?.cancel()
        listener = nil
    }

    // MARK: - HTTP

    private func handle(_ connection: NWConnection) {
        connection.start(queue: queue)
        receiveRequest(connection, buffer: Data())
    }

    private func receiveRequest(_ connection: NWConnection, buffer: Data) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 64 * 1024) { [weak self] data, _, isComplete, error in
            guard let self else { connection.cancel(); return }
            var buffer = buffer
            if let data { buffer.append(data) }
            if let end = buffer.range(of: Data("\r\n\r\n".utf8)) {
                let head = String(decoding: buffer[..<end.lowerBound], as: UTF8.self)
                self.respond(to: head, on: connection)
            } else if error != nil || isComplete || buffer.count > 64 * 1024 {
                connection.cancel()
            } else {
                self.receiveRequest(connection, buffer: buffer)
            }
        }
    }

    private func respond(to head: String, on connection: NWConnection) {
        let requestLine = head.split(separator: "\r\n", maxSplits: 1).first.map(String.init) ?? ""
        let parts = requestLine.split(separator: " ")
        guard parts.count >= 2 else { return send(status: "400 Bad Request", on: connection) }
        let method = String(parts[0])
        guard method == "GET" || method == "HEAD" else { return send(status: "405 Method Not Allowed", on: connection) }

        var path = String(parts[1])
        if let q = path.firstIndex(where: { $0 == "?" || $0 == "#" }) { path = String(path[..<q]) }
        path = path.removingPercentEncoding ?? path
        if path.hasSuffix("/") { path += "index.html" }

        let file = root.appendingPathComponent(String(path.drop(while: { $0 == "/" }))).standardizedFileURL
        // Never serve anything outside the bundled site.
        guard file.path.hasPrefix(root.path + "/"),
              let body = try? Data(contentsOf: file, options: .mappedIfSafe) else {
            return send(status: "404 Not Found", on: connection)
        }

        let cache = file.lastPathComponent == "index.html" ? "no-cache" : "public, max-age=31536000, immutable"
        var header = "HTTP/1.1 200 OK\r\n"
        header += "Content-Type: \(Self.mimeType(for: file.pathExtension))\r\n"
        header += "Content-Length: \(body.count)\r\n"
        header += "Cache-Control: \(cache)\r\n"
        header += "X-Content-Type-Options: nosniff\r\n"
        header += "Connection: close\r\n\r\n"
        var out = Data(header.utf8)
        if method == "GET" { out.append(body) }
        connection.send(content: out, completion: .contentProcessed { _ in connection.cancel() })
    }

    private func send(status: String, on connection: NWConnection) {
        let out = Data("HTTP/1.1 \(status)\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".utf8)
        connection.send(content: out, completion: .contentProcessed { _ in connection.cancel() })
    }

    private static func mimeType(for ext: String) -> String {
        switch ext.lowercased() {
        case "html": return "text/html; charset=utf-8"
        case "js", "mjs": return "text/javascript"
        case "wasm": return "application/wasm"
        case "css": return "text/css"
        case "json": return "application/json"
        case "svg": return "image/svg+xml"
        case "png": return "image/png"
        case "ttf": return "font/ttf"
        case "otf": return "font/otf"
        case "woff2": return "font/woff2"
        case "txt": return "text/plain; charset=utf-8"
        default: return "application/octet-stream"
        }
    }
}
