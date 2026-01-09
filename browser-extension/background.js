// ZOS Browser Extension - Background Service Worker
class ZosNetworkHelper {
  constructor() {
    this.zosEndpoint = 'http://localhost:8080/api/v1';
    this.connectedNodes = new Set();
    this.activeProofs = new Map();
  }

  async connectToZosNetwork() {
    try {
      const response = await fetch(`${this.zosEndpoint}/nodes`);
      const nodes = await response.json();
      console.log('🌐 Connected to ZOS network:', nodes.length, 'nodes');
      return nodes;
    } catch (error) {
      console.error('❌ Failed to connect to ZOS network:', error);
      return [];
    }
  }

  async executePlugin(pluginName, args) {
    try {
      const response = await fetch(`${this.zosEndpoint}/${pluginName}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ args })
      });
      return await response.json();
    } catch (error) {
      console.error(`❌ Plugin ${pluginName} failed:`, error);
      throw error;
    }
  }

  async generateProof(data, proofType = 'zksnark') {
    return this.executePlugin(proofType, [data]);
  }

  async verifyProof(proof, publicInputs) {
    return this.executePlugin('zksnark/verify', [proof, publicInputs]);
  }

  // Advanced UI helpers
  async enhancePage(tabId) {
    // Inject ZOS helpers into page
    await chrome.scripting.executeScript({
      target: { tabId },
      files: ['zos-widget.js']
    });
  }

  async extractSemanticData(url) {
    // Use Wikidata plugin to extract semantic information
    return this.executePlugin('wikidata', [`search:${url}`]);
  }

  async validateWithLMFDB(mathExpression) {
    // Validate mathematical expressions with LMFDB
    return this.executePlugin('lmfdb', [mathExpression]);
  }

  async checkCompliance(data, regulation) {
    // Check regulatory compliance
    const plugin = regulation === 'gdpr' ? 'regulatory/gdpr' : 'regulatory/sec';
    return this.executePlugin(plugin, [data]);
  }
}

const zosHelper = new ZosNetworkHelper();

// Background script event listeners
chrome.runtime.onInstalled.addListener(() => {
  console.log('🚀 ZOS Network Helper installed');
  zosHelper.connectToZosNetwork();
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    zosHelper.enhancePage(tabId);
  }
});

// Message handling from content scripts and popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  switch (request.action) {
    case 'executePlugin':
      zosHelper.executePlugin(request.plugin, request.args)
        .then(sendResponse)
        .catch(error => sendResponse({ error: error.message }));
      return true;

    case 'generateProof':
      zosHelper.generateProof(request.data, request.proofType)
        .then(sendResponse)
        .catch(error => sendResponse({ error: error.message }));
      return true;

    case 'extractSemantic':
      zosHelper.extractSemanticData(request.url)
        .then(sendResponse)
        .catch(error => sendResponse({ error: error.message }));
      return true;

    case 'validateMath':
      zosHelper.validateWithLMFDB(request.expression)
        .then(sendResponse)
        .catch(error => sendResponse({ error: error.message }));
      return true;

    case 'checkCompliance':
      zosHelper.checkCompliance(request.data, request.regulation)
        .then(sendResponse)
        .catch(error => sendResponse({ error: error.message }));
      return true;
  }
});
