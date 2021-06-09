use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cluster {
    Testnet,
    Mainnet,
    Devnet,
    Localnet,
    Debug,
    Custom(String, String),
}

impl Default for Cluster {
    fn default() -> Self {
        Cluster::Localnet
    }
}

impl FromStr for Cluster {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Cluster> {
        match s.to_lowercase().as_str() {
            "t" | "testnet" => Ok(Cluster::Testnet),
            "m" | "mainnet" => Ok(Cluster::Mainnet),
            "d" | "devnet" => Ok(Cluster::Devnet),
            "l" | "localnet" => Ok(Cluster::Localnet),
            "g" | "debug" => Ok(Cluster::Debug),
            url if url.contains("http") => {
                let http_url = url;

                // Websocket port is always +1 the http port.
                let mut ws_url = Url::parse(http_url)?;
                if let Some(port) = ws_url.port() {
                    ws_url.set_port(Some(port + 1))
                        .map_err(|_| anyhow!("Unable to set port"))?;
                } else {
                    ws_url.set_port(Some(8900))
                        .map_err(|_| anyhow!("Unable to set port"))?;
                }

                Ok(Cluster::Custom(http_url.to_string(), ws_url.to_string()))
            }
            _ => Err(anyhow::Error::msg(
                "Cluster must be one of [localnet, testnet, mainnet, devnet] or be an http or https url\n",
            )),
        }
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let clust_str = match self {
            Cluster::Testnet => "testnet",
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Localnet => "localnet",
            Cluster::Debug => "debug",
            Cluster::Custom(url, _ws_url) => url,
        };
        write!(f, "{}", clust_str)
    }
}

impl Cluster {
    pub fn url(&self) -> &str {
        match self {
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Testnet => "https://api.testnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Debug => "http://34.90.18.145:8899",
            Cluster::Custom(url, _ws_url) => url,
        }
    }
    pub fn ws_url(&self) -> &str {
        match self {
            Cluster::Devnet => "wss://api.devnet.solana.com",
            Cluster::Testnet => "wss://api.testnet.solana.com",
            Cluster::Mainnet => "wss://api.mainnet-beta.solana.com",
            Cluster::Localnet => "ws://127.0.0.1:9000",
            Cluster::Debug => "ws://34.90.18.145:9000",
            Cluster::Custom(_url, ws_url) => ws_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cluster(name: &str, cluster: Cluster) {
        assert_eq!(Cluster::from_str(name).unwrap(), cluster);
    }

    #[test]
    fn test_cluster_parse() {
        test_cluster("testnet", Cluster::Testnet);
        test_cluster("mainnet", Cluster::Mainnet);
        test_cluster("devnet", Cluster::Devnet);
        test_cluster("localnet", Cluster::Localnet);
        test_cluster("debug", Cluster::Debug);
    }

    #[test]
    #[should_panic]
    fn test_cluster_bad_parse() {
        let bad_url = "httq://my_custom_url.test.net";
        Cluster::from_str(bad_url).unwrap();
    }

    #[test]
    fn test_http_port() {
        let url = "http://my-url.com:7000/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "http://my-url.com:7001/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_http_no_port() {
        let url = "http://my-url.com/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "http://my-url.com:8900/".to_string()),
            cluster
        );
    }
}
