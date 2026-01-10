use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceConfiguration {
    pub id: String,
    #[serde(rename = "display-name")]
    pub display_name: String,
    #[serde(rename = "compartment-id")]
    pub compartment_id: String,
    #[serde(rename = "instance-details")]
    pub instance_details: InstanceDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceDetails {
    #[serde(rename = "launch-details")]
    pub launch_details: LaunchDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaunchDetails {
    pub shape: String,
    #[serde(rename = "source-details")]
    pub source_details: SourceDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceDetails {
    #[serde(rename = "image-id")]
    pub image_id: String,
}

pub struct OciClient {
    client: Client,
    region: String,
    tenancy_id: String,
    user_id: String,
    fingerprint: String,
    private_key: String,
}

impl OciClient {
    pub fn new(
        region: String,
        tenancy_id: String,
        user_id: String,
        fingerprint: String,
        private_key: String,
    ) -> Self {
        Self {
            client: Client::new(),
            region,
            tenancy_id,
            user_id,
            fingerprint,
            private_key,
        }
    }

    pub async fn get_instance_configuration(
        &self,
        config_id: &str,
    ) -> Result<InstanceConfiguration> {
        let url = format!(
            "https://iaas.{}.oraclecloud.com/20160918/instanceConfigurations/{}",
            self.region, config_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.create_auth_header("GET", &url)?)
            .send()
            .await?;

        let config: InstanceConfiguration = response.json().await?;
        Ok(config)
    }

    fn create_auth_header(&self, method: &str, url: &str) -> Result<String> {
        // Simplified auth - in production, implement full OCI signature
        Ok(format!("Bearer {}", base64::encode(&self.private_key)))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌩️ Oracle Cloud Rust Client for solfunmeme");

    // Configuration - replace with your actual values
    let client = OciClient::new(
        "us-ashburn-1".to_string(),
        "your-tenancy-ocid".to_string(),
        "your-user-ocid".to_string(),
        "your-fingerprint".to_string(),
        "your-private-key".to_string(),
    );

    let config_id = "ocid1.instanceconfiguration.oc1.iad.aaaaaaaaabzhyygoc5clndba7tpuskdlkl2weivohvjkl65s5cvobuvywcrq";

    match client.get_instance_configuration(config_id).await {
        Ok(config) => {
            println!("✅ Retrieved configuration:");
            println!("  Name: {}", config.display_name);
            println!("  Shape: {}", config.instance_details.launch_details.shape);
            println!(
                "  Image: {}",
                config
                    .instance_details
                    .launch_details
                    .source_details
                    .image_id
            );

            // Save to JSON
            let json = serde_json::to_string_pretty(&config)?;
            std::fs::write("solfunmeme-config.json", json)?;
            println!("📁 Saved to solfunmeme-config.json");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("💡 You need to configure OCI credentials first");
        }
    }

    Ok(())
}
