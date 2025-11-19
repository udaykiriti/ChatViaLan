<h1> Rust Chat — Real-Time WebSocket Chat Server</h1>

<p>
A fully local, offline-friendly, multi-user chat system built using <strong>Rust</strong>, 
<strong>Warp</strong>, and <strong>WebSockets</strong>.<br>
Designed for LAN/localhost use, works completely without internet, and includes file sharing,
rooms, login system, and a clean responsive UI.
</p>

<hr>

<h2> Features</h2>
<ul>
  <li>Real-time WebSocket chat</li>
  <li>Multiple rooms (<code>/join</code>, <code>/rooms</code>, <code>/leave</code>)</li>
  <li>User accounts (<code>/register</code>, <code>/login</code>) stored in <code>users.json</code></li>
  <li>Guest mode support</li>
  <li>Private messaging (<code>/msg username text</code>)</li>
  <li>File upload + sharing (stored in <code>uploads/</code>)</li>
  <li>Persistent message history per room</li>
  <li>Responsive frontend UI</li>
  <li>Works 100% offline (LAN/local)</li>
  <li>Simple project structure</li>
</ul>

<hr>

<h2> Project Flow</h2>

<pre>
rust-chat/
│
├── Cargo.toml
├── users.json            # usernames + salted SHA256 hashed passwords
├── uploads/              # stored uploaded files
├── static/
│   └── index.html        # frontend UI
└── src/
    └── main.rs           # Rust server code
</pre>

<hr>

<h2> Installation & Run</h2>

<h3>1. Install Rust</h3>
<pre><code>curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
</code></pre>

<h3>2. Run the server</h3>
<pre><code>cargo run --release
</code></pre>

<p>You should see:</p>

<pre><code>Server running at http://0.0.0.0:8080/
</code></pre>

<h3>3. Open the Chat UI</h3>
<ul>
  <li>Local machine → <a href="http://localhost:8080">http://localhost:8080</a></li>
  <li>LAN (WiFi) → <code>http://YOUR_LOCAL_IP:8080</code></li>
</ul>

<p>Find your IP:</p>
<pre><code>ip a
</code></pre>

<hr>

<h2> Chat Commands</h2>

<table border="1" cellpadding="6">
  <tr>
    <th>Command</th>
    <th>Description</th>
  </tr>
  <tr><td><code>/name &lt;newname&gt;</code></td><td>Change display name</td></tr>
  <tr><td><code>/register &lt;user&gt; &lt;pass&gt;</code></td><td>Create account</td></tr>
  <tr><td><code>/login &lt;user&gt; &lt;pass&gt;</code></td><td>Authenticate</td></tr>
  <tr><td><code>/msg &lt;user&gt; &lt;text&gt;</code></td><td>Private message</td></tr>
  <tr><td><code>/join &lt;room&gt;</code></td><td>Join/create a room</td></tr>
  <tr><td><code>/rooms</code></td><td>List rooms</td></tr>
  <tr><td><code>/leave</code></td><td>Back to lobby</td></tr>
  <tr><td><code>/history</code></td><td>Show recent messages</td></tr>
</table>

<hr>

<h2> File Upload & Sharing</h2>

<p>Click <strong>Upload & Share</strong> in the UI.<br>
Uploaded files are stored in:</p>

<pre><code>uploads/&lt;unique-id&gt;_filename.ext
</code></pre>

<p>Example link:</p>

<pre><code>http://localhost:8080/uploads/12345_file.png
</code></pre>

<hr>

<h2> Offline / LAN Usage</h2>

<p>This chat server works <strong>100% offline</strong>:</p>
<ul>
  <li>UI served from <code>static/index.html</code></li>
  <li>WebSocket: <code>ws://localhost/ws</code></li>
  <li>No CDNs or external APIs</li>
  <li>Local file storage</li>
</ul>

<p>Allow port 8080 on Linux:</p>

<pre><code>sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
</code></pre>

<hr>

<h2> Security Notes</h2>

<ul>
  <li>Passwords use SHA-256 + username salt</li>
  <li>Safe for local/LAN use</li>
  <li>Not production secure — consider Argon2</li>
</ul>

<hr>

<h2> Example users.json</h2>

<pre><code>{
  "alice": {
    "salt": "alice",
    "hash": "c6f5b1633e93c2a4a093e..."
  }
}
</code></pre>

<hr>

<h2>Future Improvements</h2>

<ul>
  <li>Argon2 password hashing</li>
  <li>Message edit/delete</li>
  <li>Typing indicators</li>
  <li>Admin/mod tools</li>
  <li>SQLite message database</li>
  <li>Dark mode</li>
</ul>

<hr>

<h2>License</h2>

<p>MIT — Free to use, modify, and redistribute.</p>

<hr>

<p>No MOree </p>
