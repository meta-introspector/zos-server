// ZOS Browser Extension Popup Script
class ZosPopup {
  constructor() {
    this.statusElement = document.getElementById('status');
    this.nodeCountElement = document.getElementById('node-count');
    this.connectionStatusElement = document.getElementById('connection-status');
    this.init();
  }

  init() {
    this.updateNetworkStatus();
    this.bindEventListeners();
  }

  async updateNetworkStatus() {
    try {
      const response = await chrome.runtime.sendMessage({
        action: 'executePlugin',
        plugin: 'libp2p',
        args: ['list_peers']
      });

      this.nodeCountElement.textContent = response.peers?.length || 0;
      this.connectionStatusElement.textContent = 'Connected';
      this.connectionStatusElement.style.color = '#4CAF50';
    } catch (error) {
      this.connectionStatusElement.textContent = 'Disconnected';
      this.connectionStatusElement.style.color = '#f44336';
    }
  }

  bindEventListeners() {
    // Zero Knowledge Proofs
    document.getElementById('generate-page-proof').onclick = () => this.generatePageProof();
    document.getElementById('verify-proof').onclick = () => this.verifyProof();
    document.getElementById('create-rollup').onclick = () => this.createRollup();

    // Semantic Analysis
    document.getElementById('extract-entities').onclick = () => this.extractEntities();
    document.getElementById('validate-math').onclick = () => this.validateMath();
    document.getElementById('check-facts').onclick = () => this.checkFacts();

    // Compliance
    document.getElementById('gdpr-check').onclick = () => this.gdprCheck();
    document.getElementById('sec-validate').onclick = () => this.secValidate();
    document.getElementById('quality-audit').onclick = () => this.qualityAudit();

    // Blockchain
    document.getElementById('ethereum-interact').onclick = () => this.ethereumInteract();
    document.getElementById('solana-interact').onclick = () => this.solanaInteract();
    document.getElementById('bitcoin-interact').onclick = () => this.bitcoinInteract();

    // System
    document.getElementById('plugin-status').onclick = () => this.pluginStatus();
    document.getElementById('node-sync').onclick = () => this.nodeSync();
    document.getElementById('settings').onclick = () => this.openSettings();
  }

  async executeZosPlugin(plugin, args = []) {
    this.setStatus('Processing...', 'info');

    try {
      const response = await chrome.runtime.sendMessage({
        action: 'executePlugin',
        plugin,
        args
      });

      this.setStatus(`✅ ${plugin} completed`, 'success');
      return response;
    } catch (error) {
      this.setStatus(`❌ ${plugin} failed: ${error.message}`, 'error');
      throw error;
    }
  }

  async generatePageProof() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    const pageData = {
      url: tab.url,
      title: tab.title,
      timestamp: Date.now()
    };

    await this.executeZosPlugin('zksnark', [JSON.stringify(pageData)]);
  }

  async verifyProof() {
    // Get proof from storage or user input
    const proof = await this.getStoredProof();
    if (proof) {
      await this.executeZosPlugin('zksnark/verify', [proof.proof, proof.publicInputs]);
    } else {
      this.setStatus('No proof to verify', 'warning');
    }
  }

  async createRollup() {
    await this.executeZosPlugin('rollup', ['create_batch']);
  }

  async extractEntities() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    await this.executeZosPlugin('wikidata', [`extract:${tab.url}`]);
  }

  async validateMath() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

    // Extract math from page
    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      function: () => {
        const mathElements = document.querySelectorAll('math, .math, [class*="equation"]');
        return Array.from(mathElements).map(el => el.textContent);
      }
    });

    if (results[0].result.length > 0) {
      await this.executeZosPlugin('lmfdb', results[0].result);
    } else {
      this.setStatus('No math expressions found', 'warning');
    }
  }

  async checkFacts() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    await this.executeZosPlugin('wikidata', [`verify:${tab.url}`]);
  }

  async gdprCheck() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

    // Extract forms and personal data
    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      function: () => {
        const forms = document.querySelectorAll('form');
        const personalData = [];

        forms.forEach(form => {
          const inputs = form.querySelectorAll('input[type="email"], input[type="tel"], input[name*="name"]');
          inputs.forEach(input => {
            if (input.value) personalData.push(input.value);
          });
        });

        return personalData;
      }
    });

    if (results[0].result.length > 0) {
      await this.executeZosPlugin('regulatory/gdpr', results[0].result);
    } else {
      this.setStatus('No personal data found', 'info');
    }
  }

  async secValidate() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

    // Look for financial data
    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      function: () => {
        const financialPatterns = [
          /\$[\d,]+\.?\d*/g,
          /revenue|profit|earnings|financial/gi,
          /\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/g
        ];

        const text = document.body.innerText;
        const matches = [];

        financialPatterns.forEach(pattern => {
          const found = text.match(pattern);
          if (found) matches.push(...found);
        });

        return matches;
      }
    });

    if (results[0].result.length > 0) {
      await this.executeZosPlugin('sec', results[0].result);
    } else {
      this.setStatus('No financial data found', 'info');
    }
  }

  async qualityAudit() {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    await this.executeZosPlugin('quality', [`audit:${tab.url}`]);
  }

  async ethereumInteract() {
    await this.executeZosPlugin('ethereum', ['get_balance', '0x...']);
  }

  async solanaInteract() {
    await this.executeZosPlugin('solana', ['get_account_info']);
  }

  async bitcoinInteract() {
    await this.executeZosPlugin('bitcoin', ['get_block_height']);
  }

  async pluginStatus() {
    const response = await this.executeZosPlugin('system', ['plugin_status']);

    // Show plugin status in a new tab
    chrome.tabs.create({
      url: chrome.runtime.getURL('plugin-status.html')
    });
  }

  async nodeSync() {
    await this.executeZosPlugin('libp2p', ['sync_nodes']);
  }

  openSettings() {
    chrome.runtime.openOptionsPage();
  }

  async getStoredProof() {
    const result = await chrome.storage.local.get(['lastProof']);
    return result.lastProof;
  }

  setStatus(message, type = 'info') {
    this.statusElement.textContent = message;
    this.statusElement.className = `status ${type}`;

    // Auto-clear status after 3 seconds
    setTimeout(() => {
      this.statusElement.textContent = 'Ready';
      this.statusElement.className = 'status';
    }, 3000);
  }
}

// Initialize popup when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  new ZosPopup();
});
