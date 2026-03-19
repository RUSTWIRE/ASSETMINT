// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
// AssetMint Sovereign Metadata Service
// Replaces OriginTrail DKG Edge Node with a self-hosted, private-by-default
// metadata store. All data stays on YOUR infrastructure.
//
// API compatible with DKG Edge Node endpoints:
//   POST /publish  — Store asset metadata, returns UAL
//   GET  /get      — Retrieve by UAL
//   GET  /info     — Service info
//   GET  /health   — Health check
//
// Storage: SQLite (sovereign, no external dependencies)
// Port: 8900 (same as DKG Edge Node)

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const PORT = process.env.PORT || 8900;
const DB_FILE = process.env.DB_FILE || '/data/metadata.json';
const LOG_PREFIX = '[K-RWA]';

// Simple JSON file store (production would use SQLite/PostgreSQL)
let store = {};

function loadStore() {
  try {
    if (fs.existsSync(DB_FILE)) {
      store = JSON.parse(fs.readFileSync(DB_FILE, 'utf8'));
      console.log(`${LOG_PREFIX} Loaded ${Object.keys(store).length} assets from ${DB_FILE}`);
    }
  } catch (e) {
    console.log(`${LOG_PREFIX} Starting with empty store`);
    store = {};
  }
}

function saveStore() {
  const dir = path.dirname(DB_FILE);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(DB_FILE, JSON.stringify(store, null, 2));
}

function generateUAL(metadata) {
  const hash = crypto.createHash('sha256')
    .update(JSON.stringify(metadata))
    .update(Date.now().toString())
    .digest('hex')
    .substring(0, 16);
  return `did:assetmint:sovereign/${hash}`;
}

function parseBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', () => {
      try { resolve(JSON.parse(body)); }
      catch (e) { reject(e); }
    });
  });
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  res.setHeader('Content-Type', 'application/json');
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  try {
    // GET /info — Service information
    if (url.pathname === '/info' && req.method === 'GET') {
      res.writeHead(200);
      res.end(JSON.stringify({
        service: 'AssetMint Sovereign Metadata',
        version: '1.0.0',
        mode: 'private',
        sovereign: true,
        assets_stored: Object.keys(store).length,
        storage: 'local-json',
        uptime_seconds: Math.floor(process.uptime()),
      }));
      return;
    }

    // GET /health — Health check
    if (url.pathname === '/health' && req.method === 'GET') {
      res.writeHead(200);
      res.end(JSON.stringify({
        status: 'healthy',
        sovereign: true,
        assets: Object.keys(store).length,
      }));
      return;
    }

    // POST /publish — Store asset metadata with verifiable hash
    if (url.pathname === '/publish' && req.method === 'POST') {
      const metadata = await parseBody(req);
      const ual = generateUAL(metadata);

      // Compute canonical metadata hash for on-chain verification
      const canonicalJson = JSON.stringify(metadata, Object.keys(metadata).sort());
      const metadataHash = crypto.createHash('sha256').update(canonicalJson).digest('hex');

      const record = {
        ual,
        metadata,
        metadata_hash: metadataHash,
        created_at: new Date().toISOString(),
        private: true,
        sovereign: true,
      };
      store[ual] = record;
      saveStore();
      console.log(`${LOG_PREFIX} Published asset: ${ual} (hash: ${metadataHash.substring(0, 16)}...)`);
      res.writeHead(201);
      res.end(JSON.stringify({
        ual,
        metadata_hash: metadataHash,
        status: 'published',
        private: true,
        verify_instruction: 'Commit metadata_hash on-chain via POST /audit/commit to make it verifiable on Kaspa DAG',
      }));
      return;
    }

    // GET /get?ual=... — Retrieve by UAL
    if (url.pathname === '/get' && req.method === 'GET') {
      const ual = url.searchParams.get('ual');
      if (!ual) {
        res.writeHead(400);
        res.end(JSON.stringify({ error: 'Missing ual parameter' }));
        return;
      }
      const record = store[ual];
      if (!record) {
        res.writeHead(404);
        res.end(JSON.stringify({ error: 'Asset not found', ual }));
        return;
      }
      res.writeHead(200);
      res.end(JSON.stringify(record));
      return;
    }

    // POST /verify — Verify metadata integrity against stored hash
    if (url.pathname === '/verify' && req.method === 'POST') {
      const body = await parseBody(req);
      const { ual, metadata } = body;
      if (!ual || !metadata) {
        res.writeHead(400);
        res.end(JSON.stringify({ error: 'Provide ual and metadata' }));
        return;
      }
      const record = store[ual];
      if (!record) {
        res.writeHead(404);
        res.end(JSON.stringify({ verified: false, error: 'Asset not found' }));
        return;
      }
      const canonicalJson = JSON.stringify(metadata, Object.keys(metadata).sort());
      const computedHash = crypto.createHash('sha256').update(canonicalJson).digest('hex');
      const matches = computedHash === record.metadata_hash;
      res.writeHead(200);
      res.end(JSON.stringify({
        verified: matches,
        ual,
        stored_hash: record.metadata_hash,
        computed_hash: computedHash,
        tampered: !matches,
      }));
      return;
    }

    // GET /assets — List all assets
    if (url.pathname === '/assets' && req.method === 'GET') {
      const assets = Object.values(store).map(r => ({
        ual: r.ual,
        name: r.metadata?.name || 'unnamed',
        created_at: r.created_at,
      }));
      res.writeHead(200);
      res.end(JSON.stringify({ count: assets.length, assets }));
      return;
    }

    // 404
    res.writeHead(404);
    res.end(JSON.stringify({ error: 'Not found', path: url.pathname }));

  } catch (e) {
    console.error(`${LOG_PREFIX} Error:`, e.message);
    res.writeHead(500);
    res.end(JSON.stringify({ error: e.message }));
  }
});

loadStore();
server.listen(PORT, '0.0.0.0', () => {
  console.log(`${LOG_PREFIX} ========================================`);
  console.log(`${LOG_PREFIX}  AssetMint Sovereign Metadata Service`);
  console.log(`${LOG_PREFIX}  Port: ${PORT}`);
  console.log(`${LOG_PREFIX}  Mode: PRIVATE (sovereign, no external deps)`);
  console.log(`${LOG_PREFIX}  Storage: ${DB_FILE}`);
  console.log(`${LOG_PREFIX} ========================================`);
});
