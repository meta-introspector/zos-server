#!/usr/bin/env python3
"""
Namecheap Dynamic DNS Client for ZOS Server
Updates DNS records automatically when IP changes
"""

import requests
import time
import sys
import os
import json
from datetime import datetime

class NamecheapDDNS:
    def __init__(self, domain, password, host="@"):
        self.domain = domain
        self.password = password
        self.host = host
        self.last_ip = None
        self.update_url = "https://dynamicdns.park-your-domain.com/update"

    def get_current_ip(self):
        """Get current public IP address"""
        try:
            # Try multiple IP services for reliability
            services = [
                "https://api.ipify.org",
                "https://icanhazip.com",
                "https://ipecho.net/plain",
                "https://checkip.amazonaws.com"
            ]

            for service in services:
                try:
                    response = requests.get(service, timeout=10)
                    if response.status_code == 200:
                        return response.text.strip()
                except:
                    continue

            raise Exception("All IP services failed")

        except Exception as e:
            print(f"❌ Error getting IP: {e}")
            return None

    def update_dns(self, ip):
        """Update Namecheap DNS record"""
        try:
            params = {
                'host': self.host,
                'domain': self.domain,
                'password': self.password,
                'ip': ip
            }

            response = requests.get(self.update_url, params=params, timeout=30)

            if response.status_code == 200:
                # Check if update was successful
                if '<ErrCount>0</ErrCount>' in response.text:
                    print(f"✅ DNS updated successfully: {self.host}.{self.domain} → {ip}")
                    return True
                else:
                    print(f"❌ DNS update failed: {response.text}")
                    return False
            else:
                print(f"❌ HTTP error {response.status_code}: {response.text}")
                return False

        except Exception as e:
            print(f"❌ Error updating DNS: {e}")
            return False

    def check_and_update(self):
        """Check IP and update if changed"""
        current_ip = self.get_current_ip()

        if not current_ip:
            print("⚠️  Could not determine current IP")
            return False

        if current_ip != self.last_ip:
            print(f"🔄 IP changed: {self.last_ip} → {current_ip}")

            if self.update_dns(current_ip):
                self.last_ip = current_ip
                self.save_state()
                return True
            else:
                return False
        else:
            print(f"✓ IP unchanged: {current_ip}")
            return True

    def save_state(self):
        """Save current state to file"""
        try:
            state = {
                'last_ip': self.last_ip,
                'last_update': datetime.now().isoformat(),
                'domain': self.domain,
                'host': self.host
            }

            with open('/tmp/namecheap_ddns_state.json', 'w') as f:
                json.dump(state, f)

        except Exception as e:
            print(f"⚠️  Could not save state: {e}")

    def load_state(self):
        """Load previous state from file"""
        try:
            with open('/tmp/namecheap_ddns_state.json', 'r') as f:
                state = json.load(f)
                self.last_ip = state.get('last_ip')
                print(f"📂 Loaded previous state: IP={self.last_ip}")

        except FileNotFoundError:
            print("📂 No previous state found, starting fresh")
        except Exception as e:
            print(f"⚠️  Could not load state: {e}")

def main():
    # Configuration from environment variables or command line
    domain = os.getenv('NAMECHEAP_DOMAIN') or sys.argv[1] if len(sys.argv) > 1 else None
    password = os.getenv('NAMECHEAP_PASSWORD') or sys.argv[2] if len(sys.argv) > 2 else None
    host = os.getenv('NAMECHEAP_HOST', '@')

    if not domain or not password:
        print("Usage: python3 namecheap_ddns.py <domain> <password> [host]")
        print("Or set environment variables:")
        print("  NAMECHEAP_DOMAIN=solfunmeme.com")
        print("  NAMECHEAP_PASSWORD=your_ddns_password")
        print("  NAMECHEAP_HOST=node1  # optional, defaults to @")
        sys.exit(1)

    print(f"🌐 Namecheap DDNS Client for {host}.{domain}")
    print(f"⏰ Started at {datetime.now()}")

    # Create DDNS client
    ddns = NamecheapDDNS(domain, password, host)
    ddns.load_state()

    # Check if running as daemon or one-shot
    daemon_mode = '--daemon' in sys.argv or os.getenv('DDNS_DAEMON') == 'true'

    if daemon_mode:
        print("🔄 Running in daemon mode (checking every 5 minutes)")

        while True:
            try:
                ddns.check_and_update()
                time.sleep(300)  # 5 minutes

            except KeyboardInterrupt:
                print("\n👋 Shutting down...")
                break
            except Exception as e:
                print(f"❌ Unexpected error: {e}")
                time.sleep(60)  # Wait 1 minute before retry
    else:
        print("🔄 Running one-shot update")
        success = ddns.check_and_update()
        sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
