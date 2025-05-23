use cloudflare::endpoints::dns::dns::{
    DnsContent, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams,
};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::blocking_api::HttpApiClient;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::{Environment, OrderDirection};
use dotenvy::dotenv;
use std::env;
use std::net::Ipv4Addr;
use std::thread::sleep;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Load environment variables.
    dotenv().ok();

    // Init logger
    env_logger::init();

    // Read relevant environment variables.
    let (email, key, zone) = (
        env::var("CLOUDFLARE_EMAIL")?,
        env::var("CLOUDFLARE_API_KEY")?,
        env::var("CLOUDFLARE_ZONE")?,
    );

    // Create Cloudflare client
    let client = HttpApiClient::new(
        Credentials::UserAuthKey {
            email: email.to_string(),
            key: key.to_string(),
        },
        ClientConfig::default(),
        Environment::Production,
    )?;

    let refresh_duration = env::var("REFRESH_DURATION")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()?;
    let refresh_duration = Duration::from_secs(refresh_duration);

    loop {
        // Obtain public Ipv4 address
        log::info!("Getting public IPv4 address...");
        let ipv4 = reqwest::blocking::get("https://api.ipify.org")?
            .text()?
            .parse::<Ipv4Addr>()?;
        log::info!("Public IPv4 address is: {}", ipv4);

        // Get all DNS records for the specified zone.
        let list_dns_endpoint = ListDnsRecords {
            zone_identifier: &zone,
            params: ListDnsRecordsParams {
                direction: Some(OrderDirection::Ascending),
                ..Default::default()
            },
        };

        log::info!("Inspecting all DNS records for zone: {}", zone);

        // Patch DNS records where the associated ipv4 address does not match our public ipv4
        client
            .request(&list_dns_endpoint)?
            .result
            .iter()
            .filter(|record| {
                if let DnsContent::A { content } = &record.content {
                    *content != ipv4
                } else {
                    false
                }
            })
            .for_each(|record| {
                log::info!("Found outdated DNS ipv4 entry for: '{}'", record.name);
                let patch_dns_endpoint = UpdateDnsRecord {
                    zone_identifier: &zone,
                    identifier: &record.id,
                    params: UpdateDnsRecordParams {
                        ttl: Some(record.ttl),
                        proxied: Some(record.proxied),
                        name: &record.name,
                        content: DnsContent::A { content: ipv4 },
                    },
                };
                match client.request(&patch_dns_endpoint) {
                    Ok(_) => log::info!("Successfully patched DNS record for: '{}'", record.name),
                    Err(e) => log::error!(
                        "Failed to patch DNS record for: '{}'. Error: {}",
                        record.name,
                        e
                    ),
                }
            });

        log::info!("Finished updating DNS records.");
        sleep(refresh_duration);
    }
}
