use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramBouncerBot {
    pub bot_token: String,
    pub linked_accounts: HashMap<i64, LinkedAccount>, // telegram_id -> account
    pub pending_links: HashMap<String, PendingLink>,  // verification_code -> link
    pub group_permissions: HashMap<i64, GroupConfig>, // chat_id -> config
    pub access_logs: HashMap<i64, Vec<AccessLog>>,    // telegram_id -> logs
    pub webhook_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedAccount {
    pub telegram_id: i64,
    pub telegram_username: Option<String>,
    pub wallet_address: String,
    pub user_id: String,
    pub linked_at: u64,
    pub verification_status: VerificationStatus,
    pub access_level: AccessLevel,
    pub reputation_score: f32,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingLink {
    pub verification_code: String,
    pub telegram_id: i64,
    pub wallet_address: String,
    pub expires_at: u64,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub chat_id: i64,
    pub group_name: String,
    pub access_requirements: AccessRequirements,
    pub auto_kick_enabled: bool,
    pub welcome_message: Option<String>,
    pub rules_message: Option<String>,
    pub admin_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequirements {
    pub min_balance: Option<u64>,
    pub required_tier: Option<String>,
    pub min_reputation: Option<f32>,
    pub required_verifications: Vec<String>,
    pub whitelist_wallets: Vec<String>,
    pub blacklist_wallets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub timestamp: u64,
    pub action: String,
    pub chat_id: i64,
    pub success: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Guest,      // Limited access
    Member,     // Full access
    VIP,        // Premium features
    Admin,      // Group management
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub new_chat_members: Option<Vec<TelegramUser>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: TelegramUser,
    pub data: Option<String>,
}

impl TelegramBouncerBot {
    pub fn new(bot_token: &str, webhook_url: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            linked_accounts: HashMap::new(),
            pending_links: HashMap::new(),
            group_permissions: HashMap::new(),
            access_logs: HashMap::new(),
            webhook_url: webhook_url.to_string(),
        }
    }

    pub fn start_wallet_linking(&mut self, telegram_id: i64, wallet_address: &str) -> Result<String, String> {
        // Generate verification code
        let verification_code = format!("VERIFY_{}",
            (telegram_id as u64 ^ chrono::Utc::now().timestamp() as u64) % 1000000);

        let pending_link = PendingLink {
            verification_code: verification_code.clone(),
            telegram_id,
            wallet_address: wallet_address.to_string(),
            expires_at: chrono::Utc::now().timestamp() as u64 + 300, // 5 minutes
            attempts: 0,
        };

        self.pending_links.insert(verification_code.clone(), pending_link);

        // Generate verification link
        let verification_link = format!("https://node1.solfunmeme.com/verify-telegram?code={}&wallet={}",
                                       verification_code, wallet_address);

        println!("🔗 Telegram linking started: TG:{} ↔ Wallet:{}", telegram_id, &wallet_address[..8]);

        Ok(format!(
            "🔐 *Wallet Verification*\n\n\
            Click this link to verify your wallet:\n\
            {}\n\n\
            ⏰ Link expires in 5 minutes\n\
            🔢 Verification code: `{}`",
            verification_link, verification_code
        ))
    }

    pub fn complete_wallet_linking(&mut self, verification_code: &str,
                                  signed_message: &str) -> Result<String, String> {

        let pending_link = self.pending_links.remove(verification_code)
            .ok_or("Invalid or expired verification code")?;

        if chrono::Utc::now().timestamp() as u64 > pending_link.expires_at {
            return Err("Verification code expired".to_string());
        }

        // Verify signature (simplified - would use actual crypto verification)
        if !self.verify_wallet_signature(&pending_link.wallet_address, signed_message, verification_code) {
            return Err("Invalid wallet signature".to_string());
        }

        // Create linked account
        let linked_account = LinkedAccount {
            telegram_id: pending_link.telegram_id,
            telegram_username: None, // Will be updated from Telegram API
            wallet_address: pending_link.wallet_address.clone(),
            user_id: format!("tg_{}", pending_link.telegram_id),
            linked_at: chrono::Utc::now().timestamp() as u64,
            verification_status: VerificationStatus::Verified,
            access_level: AccessLevel::Member,
            reputation_score: 50.0,
            last_activity: chrono::Utc::now().timestamp() as u64,
        };

        self.linked_accounts.insert(pending_link.telegram_id, linked_account);

        println!("✅ Telegram account linked: TG:{} ↔ Wallet:{}",
                 pending_link.telegram_id, &pending_link.wallet_address[..8]);

        Ok("✅ Wallet successfully linked to your Telegram account!".to_string())
    }

    pub fn handle_telegram_update(&mut self, update: TelegramUpdate) -> Result<Vec<TelegramResponse>, String> {
        let mut responses = Vec::new();

        // Handle new chat members
        if let Some(message) = &update.message {
            if let Some(new_members) = &message.new_chat_members {
                for member in new_members {
                    if !member.is_bot {
                        let response = self.handle_new_member(member, &message.chat)?;
                        responses.push(response);
                    }
                }
            }

            // Handle commands
            if let Some(text) = &message.text {
                if text.starts_with('/') {
                    let response = self.handle_command(text, message)?;
                    responses.push(response);
                }
            }
        }

        // Handle callback queries (inline buttons)
        if let Some(callback) = &update.callback_query {
            let response = self.handle_callback_query(callback)?;
            responses.push(response);
        }

        Ok(responses)
    }

    fn handle_new_member(&mut self, member: &TelegramUser, chat: &TelegramChat) -> Result<TelegramResponse, String> {
        let group_config = self.group_permissions.get(&chat.id);

        // Check if user has linked wallet
        if let Some(linked_account) = self.linked_accounts.get(&member.id) {
            // Check access requirements
            if let Some(config) = group_config {
                let access_granted = self.check_access_requirements(linked_account, &config.access_requirements)?;

                if access_granted {
                    self.log_access(member.id, chat.id, "join_approved", true, None);

                    let welcome_msg = config.welcome_message.as_deref()
                        .unwrap_or("Welcome! Your wallet is verified ✅");

                    return Ok(TelegramResponse::SendMessage {
                        chat_id: chat.id,
                        text: format!("👋 Welcome {}!\n\n{}", member.first_name, welcome_msg),
                        reply_markup: None,
                    });
                } else {
                    // Kick user - insufficient access
                    self.log_access(member.id, chat.id, "join_denied", false, Some("Insufficient wallet balance/tier".to_string()));

                    return Ok(TelegramResponse::KickChatMember {
                        chat_id: chat.id,
                        user_id: member.id,
                        reason: "Wallet does not meet group requirements".to_string(),
                    });
                }
            }
        } else {
            // No linked wallet - provide linking instructions
            self.log_access(member.id, chat.id, "join_pending", false, Some("No linked wallet".to_string()));

            let keyboard = vec![vec![
                InlineKeyboardButton {
                    text: "🔗 Link Wallet".to_string(),
                    callback_data: Some(format!("link_wallet_{}", member.id)),
                }
            ]];

            return Ok(TelegramResponse::SendMessage {
                chat_id: chat.id,
                text: format!(
                    "👋 Welcome {}!\n\n\
                    🔐 This group requires wallet verification.\n\
                    Click the button below to link your wallet.\n\n\
                    ⏰ You have 5 minutes to verify or you'll be removed.",
                    member.first_name
                ),
                reply_markup: Some(InlineKeyboardMarkup { inline_keyboard: keyboard }),
            });
        }

        Ok(TelegramResponse::SendMessage {
            chat_id: chat.id,
            text: "Welcome!".to_string(),
            reply_markup: None,
        })
    }

    fn handle_command(&mut self, text: &str, message: &TelegramMessage) -> Result<TelegramResponse, String> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let command = parts[0];

        match command {
            "/start" => {
                Ok(TelegramResponse::SendMessage {
                    chat_id: message.chat.id,
                    text: "🤖 *ZOS Bouncer Bot*\n\n\
                           Commands:\n\
                           /link - Link your wallet\n\
                           /status - Check your verification status\n\
                           /balance - Check wallet balance\n\
                           /help - Show this help".to_string(),
                    reply_markup: None,
                })
            },
            "/link" => {
                if parts.len() < 2 {
                    return Ok(TelegramResponse::SendMessage {
                        chat_id: message.chat.id,
                        text: "Usage: /link <wallet_address>".to_string(),
                        reply_markup: None,
                    });
                }

                let wallet_address = parts[1];
                let user_id = message.from.as_ref().unwrap().id;

                match self.start_wallet_linking(user_id, wallet_address) {
                    Ok(verification_msg) => {
                        Ok(TelegramResponse::SendMessage {
                            chat_id: message.chat.id,
                            text: verification_msg,
                            reply_markup: None,
                        })
                    },
                    Err(e) => {
                        Ok(TelegramResponse::SendMessage {
                            chat_id: message.chat.id,
                            text: format!("❌ Error: {}", e),
                            reply_markup: None,
                        })
                    }
                }
            },
            "/status" => {
                let user_id = message.from.as_ref().unwrap().id;

                if let Some(account) = self.linked_accounts.get(&user_id) {
                    Ok(TelegramResponse::SendMessage {
                        chat_id: message.chat.id,
                        text: format!(
                            "✅ *Verification Status*\n\n\
                            Wallet: `{}`\n\
                            Status: {:?}\n\
                            Access Level: {:?}\n\
                            Reputation: {:.1}\n\
                            Linked: {}",
                            account.wallet_address,
                            account.verification_status,
                            account.access_level,
                            account.reputation_score,
                            chrono::DateTime::from_timestamp(account.linked_at as i64, 0)
                                .unwrap_or_default()
                                .format("%Y-%m-%d %H:%M UTC")
                        ),
                        reply_markup: None,
                    })
                } else {
                    Ok(TelegramResponse::SendMessage {
                        chat_id: message.chat.id,
                        text: "❌ No wallet linked. Use /link <wallet_address>".to_string(),
                        reply_markup: None,
                    })
                }
            },
            _ => {
                Ok(TelegramResponse::SendMessage {
                    chat_id: message.chat.id,
                    text: "Unknown command. Use /help for available commands.".to_string(),
                    reply_markup: None,
                })
            }
        }
    }

    fn handle_callback_query(&mut self, callback: &CallbackQuery) -> Result<TelegramResponse, String> {
        if let Some(data) = &callback.data {
            if data.starts_with("link_wallet_") {
                let user_id: i64 = data.replace("link_wallet_", "").parse()
                    .map_err(|_| "Invalid callback data")?;

                return Ok(TelegramResponse::SendMessage {
                    chat_id: callback.from.id,
                    text: "🔗 To link your wallet, send:\n/link <your_wallet_address>".to_string(),
                    reply_markup: None,
                });
            }
        }

        Ok(TelegramResponse::AnswerCallbackQuery {
            callback_query_id: callback.id.clone(),
            text: Some("Processing...".to_string()),
        })
    }

    fn check_access_requirements(&self, account: &LinkedAccount,
                                requirements: &AccessRequirements) -> Result<bool, String> {

        // Check minimum balance (would integrate with actual wallet balance check)
        if let Some(min_balance) = requirements.min_balance {
            // Simplified - would check actual wallet balance
            if min_balance > 1000 {
                return Ok(false);
            }
        }

        // Check reputation
        if let Some(min_reputation) = requirements.min_reputation {
            if account.reputation_score < min_reputation {
                return Ok(false);
            }
        }

        // Check whitelist/blacklist
        if !requirements.whitelist_wallets.is_empty() {
            if !requirements.whitelist_wallets.contains(&account.wallet_address) {
                return Ok(false);
            }
        }

        if requirements.blacklist_wallets.contains(&account.wallet_address) {
            return Ok(false);
        }

        Ok(true)
    }

    fn verify_wallet_signature(&self, wallet_address: &str, signature: &str, message: &str) -> bool {
        // Simplified signature verification
        // In real implementation, would verify cryptographic signature
        signature.len() > 10 && signature.contains(wallet_address) && signature.contains(message)
    }

    fn log_access(&mut self, telegram_id: i64, chat_id: i64, action: &str, success: bool, reason: Option<String>) {
        let log = AccessLog {
            timestamp: chrono::Utc::now().timestamp() as u64,
            action: action.to_string(),
            chat_id,
            success,
            reason,
        };

        self.access_logs.entry(telegram_id)
            .or_insert_with(Vec::new)
            .push(log);
    }

    pub fn configure_group(&mut self, chat_id: i64, config: GroupConfig) {
        self.group_permissions.insert(chat_id, config);
        println!("⚙️  Group configured: {}", chat_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelegramResponse {
    SendMessage {
        chat_id: i64,
        text: String,
        reply_markup: Option<InlineKeyboardMarkup>,
    },
    KickChatMember {
        chat_id: i64,
        user_id: i64,
        reason: String,
    },
    AnswerCallbackQuery {
        callback_query_id: String,
        text: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: Option<String>,
}
