// ZOS Content Script - Inject UI helpers into web pages
(function() {
  'use strict';

  class ZosPageEnhancer {
    constructor() {
      this.widgets = new Map();
      this.init();
    }

    init() {
      this.injectZosWidget();
      this.enhanceMathElements();
      this.enhanceDataElements();
      this.addComplianceCheckers();
    }

    injectZosWidget() {
      // Create floating ZOS widget
      const widget = document.createElement('div');
      widget.id = 'zos-widget';
      widget.innerHTML = `
        <div class="zos-widget-header">🔮 ZOS Network</div>
        <div class="zos-widget-content">
          <button id="zos-prove-page">Generate Page Proof</button>
          <button id="zos-verify-data">Verify Data</button>
          <button id="zos-semantic-extract">Extract Semantics</button>
          <button id="zos-compliance-check">Check Compliance</button>
        </div>
      `;
      
      widget.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        width: 200px;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        border-radius: 10px;
        padding: 10px;
        z-index: 10000;
        font-family: 'Segoe UI', sans-serif;
        box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        cursor: move;
      `;

      document.body.appendChild(widget);
      this.addWidgetEventListeners();
    }

    addWidgetEventListeners() {
      document.getElementById('zos-prove-page').onclick = () => this.generatePageProof();
      document.getElementById('zos-verify-data').onclick = () => this.verifyPageData();
      document.getElementById('zos-semantic-extract').onclick = () => this.extractSemantics();
      document.getElementById('zos-compliance-check').onclick = () => this.checkCompliance();
    }

    async generatePageProof() {
      const pageData = {
        url: window.location.href,
        title: document.title,
        content: document.body.innerText.substring(0, 1000),
        timestamp: Date.now()
      };

      try {
        const response = await chrome.runtime.sendMessage({
          action: 'generateProof',
          data: JSON.stringify(pageData),
          proofType: 'zksnark'
        });

        this.showNotification('✅ Page proof generated', response.result);
      } catch (error) {
        this.showNotification('❌ Proof generation failed', error.message);
      }
    }

    async verifyPageData() {
      const forms = document.querySelectorAll('form');
      for (const form of forms) {
        const formData = new FormData(form);
        const data = Object.fromEntries(formData.entries());
        
        try {
          const response = await chrome.runtime.sendMessage({
            action: 'executePlugin',
            plugin: 'quality',
            args: [JSON.stringify(data)]
          });

          this.highlightElement(form, response.valid ? 'green' : 'red');
        } catch (error) {
          console.error('Form validation failed:', error);
        }
      }
    }

    async extractSemantics() {
      try {
        const response = await chrome.runtime.sendMessage({
          action: 'extractSemantic',
          url: window.location.href
        });

        // Highlight semantic entities on page
        if (response.entities) {
          response.entities.forEach(entity => {
            this.highlightText(entity.text, 'yellow');
          });
        }

        this.showNotification('🧠 Semantic data extracted', `Found ${response.entities?.length || 0} entities`);
      } catch (error) {
        this.showNotification('❌ Semantic extraction failed', error.message);
      }
    }

    async checkCompliance() {
      const personalData = this.detectPersonalData();
      
      if (personalData.length > 0) {
        try {
          const response = await chrome.runtime.sendMessage({
            action: 'checkCompliance',
            data: JSON.stringify(personalData),
            regulation: 'gdpr'
          });

          personalData.forEach(element => {
            this.highlightElement(element, response.compliant ? 'green' : 'red');
          });

          this.showNotification('🛡️ GDPR compliance checked', response.compliant ? 'Compliant' : 'Issues found');
        } catch (error) {
          this.showNotification('❌ Compliance check failed', error.message);
        }
      }
    }

    enhanceMathElements() {
      // Find mathematical expressions and add LMFDB validation
      const mathElements = document.querySelectorAll('math, .math, [class*="equation"]');
      
      mathElements.forEach(element => {
        element.addEventListener('click', async () => {
          const mathText = element.textContent;
          
          try {
            const response = await chrome.runtime.sendMessage({
              action: 'validateMath',
              expression: mathText
            });

            this.showTooltip(element, response.valid ? '✅ Valid' : '❌ Invalid');
          } catch (error) {
            this.showTooltip(element, '❌ Validation failed');
          }
        });
      });
    }

    enhanceDataElements() {
      // Add proof generation to data tables
      const tables = document.querySelectorAll('table');
      
      tables.forEach(table => {
        const button = document.createElement('button');
        button.textContent = '🔮 Generate ZK Proof';
        button.style.cssText = 'margin: 5px; padding: 5px 10px; background: #667eea; color: white; border: none; border-radius: 5px;';
        
        button.onclick = async () => {
          const tableData = this.extractTableData(table);
          
          try {
            const response = await chrome.runtime.sendMessage({
              action: 'generateProof',
              data: JSON.stringify(tableData),
              proofType: 'zkstark'
            });

            this.showNotification('✅ Table proof generated', 'Data integrity verified');
          } catch (error) {
            this.showNotification('❌ Proof generation failed', error.message);
          }
        };

        table.parentNode.insertBefore(button, table);
      });
    }

    detectPersonalData() {
      const personalDataPatterns = [
        /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, // Email
        /\b\d{3}-\d{2}-\d{4}\b/g, // SSN
        /\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/g // Credit card
      ];

      const elements = [];
      personalDataPatterns.forEach(pattern => {
        const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
        let node;
        while (node = walker.nextNode()) {
          if (pattern.test(node.textContent)) {
            elements.push(node.parentElement);
          }
        }
      });

      return elements;
    }

    highlightElement(element, color) {
      element.style.border = `2px solid ${color}`;
      element.style.boxShadow = `0 0 5px ${color}`;
    }

    highlightText(text, color) {
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
      let node;
      while (node = walker.nextNode()) {
        if (node.textContent.includes(text)) {
          const span = document.createElement('span');
          span.style.backgroundColor = color;
          span.textContent = text;
          node.textContent = node.textContent.replace(text, '');
          node.parentNode.insertBefore(span, node.nextSibling);
        }
      }
    }

    showNotification(title, message) {
      const notification = document.createElement('div');
      notification.innerHTML = `<strong>${title}</strong><br>${message}`;
      notification.style.cssText = `
        position: fixed;
        top: 80px;
        right: 20px;
        background: #333;
        color: white;
        padding: 10px;
        border-radius: 5px;
        z-index: 10001;
        max-width: 300px;
      `;
      
      document.body.appendChild(notification);
      setTimeout(() => notification.remove(), 5000);
    }

    showTooltip(element, text) {
      const tooltip = document.createElement('div');
      tooltip.textContent = text;
      tooltip.style.cssText = `
        position: absolute;
        background: #333;
        color: white;
        padding: 5px;
        border-radius: 3px;
        font-size: 12px;
        z-index: 10002;
      `;
      
      element.appendChild(tooltip);
      setTimeout(() => tooltip.remove(), 3000);
    }

    extractTableData(table) {
      const data = [];
      const rows = table.querySelectorAll('tr');
      
      rows.forEach(row => {
        const cells = row.querySelectorAll('td, th');
        const rowData = Array.from(cells).map(cell => cell.textContent.trim());
        if (rowData.length > 0) data.push(rowData);
      });
      
      return data;
    }
  }

  // Initialize ZOS page enhancer
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => new ZosPageEnhancer());
  } else {
    new ZosPageEnhancer();
  }
})();
