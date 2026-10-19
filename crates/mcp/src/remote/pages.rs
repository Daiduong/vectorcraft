use super::{config::Settings, oauth::Pending};
pub fn escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}
pub fn consent(settings: &Settings, id: &str, pending: &Pending, error: Option<&str>) -> String {
    let error = error.map(|e| format!("<p class=error>{}</p>", escape(e))).unwrap_or_default();
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Connect to VectorCraft</title>
<style>body{{font:17px system-ui,sans-serif;max-width:570px;margin:5vh auto;padding:24px;line-height:1.5;color:#182128;background:#fafbfc}}main{{border:1px solid #dbe1e6;background:white;border-radius:14px;padding:28px}}h1{{margin-top:0}}input,button{{font:inherit;padding:12px;width:100%;box-sizing:border-box;margin:8px 0;border:1px solid #9ba7af;border-radius:6px}}button{{cursor:pointer}}.primary{{background:#17634f;color:white;border-color:#17634f}}small{{display:block;overflow-wrap:anywhere;color:#52616c}}.error{{color:#a21b27}}</style></head><body><main>
<h1>Connect to VectorCraft</h1><p>Approve the connection you just started in your AI app.</p><p>This grants control of the shared desktop workspace and access to its approved artwork folder.</p>
<p><strong>{}</strong><small>Return address: {}</small></p>{}
<form method="post" action="{}/approve"><input type="hidden" name="request" value="{}"><label>Owner passphrase<input name="passphrase" type="password" maxlength="512" autocomplete="current-password" required autofocus></label><button class="primary" type="submit" name="decision" value="allow">Authorize VectorCraft access</button><button type="submit" name="decision" value="deny" formnovalidate>Cancel</button></form>
<small>Set up and revoke connections in VectorCraft's Remote AI connections settings.</small></main></body></html>"#,
        escape(&pending.client_name),
        escape(&pending.redirect),
        error,
        settings.base_path,
        escape(id)
    )
}
pub fn csp(redirect: &str) -> String {
    let origin = url::Url::parse(redirect).map(|u| u.origin().ascii_serialization()).unwrap_or_default();
    // Chromium checks every redirect hop, including the Google relay returning to its AI app.
    format!(
        "default-src 'none'; style-src 'unsafe-inline'; form-action 'self' {origin} https://gemini.google.com; frame-ancestors 'none'; base-uri 'none'"
    )
}
