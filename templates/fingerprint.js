// Client-side fingerprinting JavaScript
// Include this in your web frontend

class ZOSFingerprinter {
    constructor() {
        this.fingerprint = {};
        this.responseData = {
            keystrokes: [],
            mouseMovements: [],
            clicks: [],
            scrolls: []
        };
    }

    async generateFingerprint() {
        // Device fingerprinting
        this.fingerprint.device = {
            userAgent: navigator.userAgent,
            screenResolution: `${screen.width}x${screen.height}`,
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            language: navigator.language,
            platform: navigator.platform,
            webglVendor: this.getWebGLVendor(),
            canvasHash: this.getCanvasFingerprint(),
            audioHash: await this.getAudioFingerprint(),
            fontListHash: this.getFontFingerprint()
        };

        // Start response pattern collection
        this.startResponseTracking();

        return this.fingerprint;
    }

    getWebGLVendor() {
        try {
            const canvas = document.createElement('canvas');
            const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
            const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
            return gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL);
        } catch (e) {
            return 'unknown';
        }
    }

    getCanvasFingerprint() {
        try {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            ctx.textBaseline = 'top';
            ctx.font = '14px Arial';
            ctx.fillText('ZOS Fingerprint 🔐', 2, 2);
            return this.hashString(canvas.toDataURL());
        } catch (e) {
            return 'unknown';
        }
    }

    async getAudioFingerprint() {
        try {
            const audioContext = new (window.AudioContext || window.webkitAudioContext)();
            const oscillator = audioContext.createOscillator();
            const analyser = audioContext.createAnalyser();
            const gainNode = audioContext.createGain();

            oscillator.connect(analyser);
            analyser.connect(gainNode);
            gainNode.connect(audioContext.destination);

            oscillator.frequency.value = 1000;
            gainNode.gain.value = 0;

            oscillator.start();

            const frequencyData = new Uint8Array(analyser.frequencyBinCount);
            analyser.getByteFrequencyData(frequencyData);

            oscillator.stop();
            audioContext.close();

            return this.hashString(Array.from(frequencyData).join(''));
        } catch (e) {
            return 'unknown';
        }
    }

    getFontFingerprint() {
        const testFonts = [
            'Arial', 'Helvetica', 'Times New Roman', 'Courier New',
            'Verdana', 'Georgia', 'Palatino', 'Garamond',
            'Comic Sans MS', 'Trebuchet MS', 'Arial Black', 'Impact'
        ];

        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        const availableFonts = [];

        testFonts.forEach(font => {
            ctx.font = `12px ${font}`;
            const width = ctx.measureText('ZOS').width;
            if (width > 0) {
                availableFonts.push(font);
            }
        });

        return this.hashString(availableFonts.join(','));
    }

    startResponseTracking() {
        // Keystroke timing
        let lastKeyTime = 0;
        document.addEventListener('keydown', (e) => {
            const now = Date.now();
            if (lastKeyTime > 0) {
                this.responseData.keystrokes.push(now - lastKeyTime);
            }
            lastKeyTime = now;
        });

        // Mouse movement patterns
        let mousePoints = [];
        document.addEventListener('mousemove', (e) => {
            mousePoints.push([e.clientX, e.clientY, Date.now()]);
            if (mousePoints.length > 100) {
                mousePoints = mousePoints.slice(-50); // Keep last 50 points
            }
        });

        // Click patterns
        let lastClickTime = 0;
        document.addEventListener('click', (e) => {
            const now = Date.now();
            if (lastClickTime > 0) {
                this.responseData.clicks.push(now - lastClickTime);
            }
            lastClickTime = now;
        });

        // Scroll behavior
        let scrollData = [];
        document.addEventListener('scroll', (e) => {
            scrollData.push([window.scrollY, Date.now()]);
            if (scrollData.length > 50) {
                scrollData = scrollData.slice(-25);
            }
        });

        // Update response fingerprint every 5 seconds
        setInterval(() => {
            this.updateResponseFingerprint();
        }, 5000);
    }

    updateResponseFingerprint() {
        const response = {
            typingPattern: this.responseData.keystrokes.slice(-20), // Last 20 keystrokes
            mouseMovement: this.hashString(JSON.stringify(this.responseData.mouseMovements.slice(-20))),
            clickPattern: this.responseData.clicks.slice(-10), // Last 10 clicks
            scrollBehavior: this.hashString(JSON.stringify(this.responseData.scrolls.slice(-10))),
            interactionRhythm: this.calculateInteractionRhythm()
        };

        // Send to server
        fetch('/api/fingerprint/response', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(response)
        });
    }

    calculateInteractionRhythm() {
        const allTimings = [
            ...this.responseData.keystrokes,
            ...this.responseData.clicks
        ];

        if (allTimings.length < 5) return 0.0;

        const avg = allTimings.reduce((a, b) => a + b, 0) / allTimings.length;
        const variance = allTimings.reduce((sum, timing) => sum + Math.pow(timing - avg, 2), 0) / allTimings.length;

        // Return consistency score (0-1, higher = more consistent)
        return Math.max(0, 1 - (Math.sqrt(variance) / avg));
    }

    hashString(str) {
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash; // Convert to 32-bit integer
        }
        return Math.abs(hash).toString(16);
    }

    // Verification methods
    async verifyEmail(email) {
        const response = await fetch('/api/verify/email', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email })
        });
        return response.json();
    }

    async verifyTwitter(handle) {
        const response = await fetch('/api/verify/twitter', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ twitter_handle: handle })
        });
        return response.json();
    }

    async acceptLoyaltyCookies() {
        const response = await fetch('/api/loyalty/cookies', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        });

        if (response.ok) {
            const data = await response.json();
            // Store JWT loyalty key in cookie
            document.cookie = `zos_loyalty=${data.jwt_key}; path=/; max-age=31536000; secure; samesite=strict`;
            return data;
        }

        throw new Error('Failed to accept loyalty cookies');
    }

    async getUserStatus() {
        const response = await fetch('/api/user/status');
        return response.json();
    }
}

// Usage example
const fingerprinter = new ZOSFingerprinter();

// Initialize fingerprinting
fingerprinter.generateFingerprint().then(() => {
    console.log('🔐 User fingerprinted');
});

// Verification UI helpers
function showVerificationModal() {
    const modal = document.createElement('div');
    modal.innerHTML = `
        <div class="verification-modal">
            <h3>🎯 Earn More Credits</h3>
            <div class="verification-options">
                <button onclick="verifyEmail()">📧 Verify Email (+50 points)</button>
                <button onclick="verifyTwitter()">🐦 Verify Twitter (+75 points)</button>
                <button onclick="acceptCookies()">🍪 Accept Loyalty Cookies (+25 points)</button>
                <button onclick="startKYC()">🆔 Complete KYC (+500 points)</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
}

async function verifyEmail() {
    const email = prompt('Enter your email:');
    if (email) {
        const result = await fingerprinter.verifyEmail(email);
        alert(`✅ Email verified! +${result.points} points`);
    }
}

async function verifyTwitter() {
    const handle = prompt('Enter your Twitter handle:');
    if (handle) {
        const result = await fingerprinter.verifyTwitter(handle);
        alert(`✅ Twitter verified! +${result.points} points`);
    }
}

async function acceptCookies() {
    try {
        const result = await fingerprinter.acceptLoyaltyCookies();
        alert(`✅ Loyalty cookies accepted! +${result.bonus} points`);
    } catch (e) {
        alert('❌ Failed to accept cookies');
    }
}
