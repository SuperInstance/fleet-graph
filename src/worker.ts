interface Env {
  FLEET_GRAPH: KVNamespace;
}

interface Node {
  id: string;
  type: string;
  attributes: Record<string, any>;
  timestamp: number;
}

interface Edge {
  from: string;
  to: string;
  type: string;
  weight: number;
  attributes: Record<string, any>;
  timestamp: number;
}

interface Pattern {
  id: string;
  nodes: string[];
  edges: string[];
  support: number;
  confidence: number;
  timestamp: number;
}

const HTML_TEMPLATE = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Fleet Graph</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
      background: #0a0a0f;
      color: #e0e0f0;
      line-height: 1.6;
      min-height: 100vh;
      padding: 20px;
    }
    .container {
      max-width: 1200px;
      margin: 0 auto;
    }
    header {
      border-bottom: 2px solid #8b5cf6;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }
    h1 {
      color: #8b5cf6;
      font-size: 2.5rem;
      margin-bottom: 10px;
    }
    .subtitle {
      color: #a0a0c0;
      font-size: 1.1rem;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 25px;
      margin-bottom: 40px;
    }
    .card {
      background: #151520;
      border-radius: 12px;
      padding: 25px;
      border: 1px solid #2a2a3a;
      transition: transform 0.3s, border-color 0.3s;
    }
    .card:hover {
      transform: translateY(-5px);
      border-color: #8b5cf6;
    }
    h2 {
      color: #8b5cf6;
      margin-bottom: 15px;
      font-size: 1.5rem;
    }
    .endpoint {
      background: #1a1a2a;
      padding: 15px;
      border-radius: 8px;
      margin-bottom: 15px;
      border-left: 4px solid #8b5cf6;
    }
    code {
      background: #0a0a0f;
      padding: 3px 6px;
      border-radius: 4px;
      font-family: 'Courier New', monospace;
      color: #8b5cf6;
    }
    .stats {
      display: flex;
      gap: 20px;
      flex-wrap: wrap;
      margin-top: 20px;
    }
    .stat {
      background: #1a1a2a;
      padding: 15px;
      border-radius: 8px;
      flex: 1;
      min-width: 150px;
      text-align: center;
    }
    .stat-value {
      font-size: 2rem;
      color: #8b5cf6;
      font-weight: bold;
    }
    .stat-label {
      color: #a0a0c0;
      font-size: 0.9rem;
      margin-top: 5px;
    }
    footer {
      margin-top: 40px;
      text-align: center;
      color: #666680;
      padding-top: 20px;
      border-top: 1px solid #2a2a3a;
      font-size: 0.9rem;
    }
    .accent { color: #8b5cf6; }
    .health {
      display: inline-block;
      width: 12px;
      height: 12px;
      background: #10b981;
      border-radius: 50%;
      margin-right: 8px;
      animation: pulse 2s infinite;
    }
    @keyframes pulse {
      0% { opacity: 1; }
      50% { opacity: 0.5; }
      100% { opacity: 1; }
    }
    @media (max-width: 768px) {
      .grid { grid-template-columns: 1fr; }
      h1 { font-size: 2rem; }
    }
  </style>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
</head>
<body>
  <div class="container">
    <header>
      <h1>Fleet Graph</h1>
      <p class="subtitle">Lightweight in-KV graph DB for fleet-wide pattern mining</p>
      <div class="stats">
        <div class="stat">
          <div class="stat-value" id="nodeCount">0</div>
          <div class="stat-label">Nodes</div>
        </div>
        <div class="stat">
          <div class="stat-value" id="edgeCount">0</div>
          <div class="stat-label">Edges</div>
        </div>
        <div class="stat">
          <div class="stat-value" id="patternCount">0</div>
          <div class="stat-label">Patterns</div>
        </div>
      </div>
    </header>
    
    <div class="grid">
      <div class="card">
        <h2>API Endpoints</h2>
        <div class="endpoint">
          <strong>POST /api/node</strong><br>
          Add a node to the graph
        </div>
        <div class="endpoint">
          <strong>POST /api/edge</strong><br>
          Add an edge between nodes
        </div>
        <div class="endpoint">
          <strong>GET /api/patterns</strong><br>
          Retrieve mined patterns
        </div>
        <div class="endpoint">
          <strong>GET /health</strong><br>
          <span class="health"></span> Service health check
        </div>
      </div>
      
      <div class="card">
        <h2>Features</h2>
        <ul style="padding-left: 20px; color: #c0c0e0;">
          <li style="margin-bottom: 10px;">In-KV adjacency list storage</li>
          <li style="margin-bottom: 10px;">Real-time pattern detection</li>
          <li style="margin-bottom: 10px;">Cross-vessel epiphanies</li>
          <li style="margin-bottom: 10px;">Periodic mining scheduler</li>
          <li style="margin-bottom: 10px;">GNN-ready data structures</li>
          <li>Zero external dependencies</li>
        </ul>
      </div>
      
      <div class="card">
        <h2>Quick Start</h2>
        <p style="margin-bottom: 15px; color: #c0c0e0;">
          Add a node:
        </p>
        <code>curl -X POST /api/node -H "Content-Type: application/json" -d '{"id":"vessel1","type":"ship","attributes":{"lat":45.5,"lon":-122.6}}'</code>
        <p style="margin: 15px 0; color: #c0c0e0;">
          Add an edge:
        </p>
        <code>curl -X POST /api/edge -H "Content-Type: application/json" -d '{"from":"vessel1","to":"vessel2","type":"proximity","weight":0.85}'</code>
      </div>
    </div>
    
    <footer>
      <p>Fleet Graph &copy; ${new Date().getFullYear()} — GNN patterns without external databases</p>
      <p style="margin-top: 10px; font-size: 0.8rem;">Built with Cloudflare Workers & KV • Dark theme #0a0a0f • Accent #8b5cf6</p>
    </footer>
  </div>
  
  <script>
    async function updateStats() {
      try {
        const patterns = await fetch('/api/patterns').then(r => r.json());
        document.getElementById('patternCount').textContent = patterns.length || 0;
      } catch (e) {
        console.log('Stats update:', e.message);
      }
    }
    updateStats();
    setInterval(updateStats, 30000);
  </script>
</body>
</html>`;
const sh = {'Content-Security-Policy': "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; frame-ancestors 'none'",'X-Frame-Options':'DENY'};
export default { async fetch(r: Request) { const u = new URL(r.url); if (u.pathname==='/health') return new Response(JSON.stringify({status:'ok'}),{headers:{'Content-Type':'application/json',...sh}}); return new Response(html,{headers:{'Content-Type':'text/html;charset=UTF-8',...sh}}); }};