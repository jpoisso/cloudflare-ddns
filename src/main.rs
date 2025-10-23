use cloudflare::endpoints::dns::dns::{
    DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord,
    UpdateDnsRecordParams,
};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::client::blocking_api::HttpApiClient;
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
        .parse::<u64>()
        .map(Duration::from_secs)?;

    let list_dns_endpoint = ListDnsRecords {
        zone_identifier: &zone,
        params: ListDnsRecordsParams {
            direction: Some(OrderDirection::Ascending),
            ..Default::default()
        },
    };

    loop {
        // Obtain public Ipv4 address
        log::info!("Getting public IPv4 address...");
        let ipv4 = match get_public_ipv4() {
            Ok(ipv4) => ipv4,
            Err(e) => {
                log::error!("Failed to fetch public IPv4 address: {e:?}");
                sleep(refresh_duration);
                continue;
            }
        };
        log::info!("Public IPv4 address is: {}", ipv4);

        log::info!("Inspecting all DNS records for zone: {}", zone);
        let dns_records: Vec<DnsRecord> = match get_dns_records(&client, &list_dns_endpoint) {
            Ok(records) => records,
            Err(e) => {
                log::error!("Failed to fetch DNS records: {e:?}");
                sleep(refresh_duration);
                continue;
            }
        };
        let dns_records = dns_records.iter().filter(|record| {
            if let DnsContent::A { content } = &record.content {
                *content != ipv4
            } else {
                false
            }
        });

        dns_records.for_each(|record| {
            log::info!("Found outdated DNS ipv4 entry for: '{}'", record.name);
            let patch_dns_endpoint = get_patch_dns_endpoint(&zone, record, ipv4);
            match client.request(&patch_dns_endpoint) {
                Ok(_) => log::info!("Successfully patched DNS record for: '{}'", record.name),
                Err(e) => {
                    log::error!(
                        "Failed to patch DNS record for: '{}'. Error: {e:?}",
                        record.name
                    )
                }
            }
        });

        log::info!("Finished updating DNS records.");
        sleep(refresh_duration);
    }
}

fn get_public_ipv4() -> anyhow::Result<Ipv4Addr> {
    let response = reqwest::blocking::get("https://api.ipify.org")?;
    let ip_str = response.text()?;
    let ipv4 = ip_str.parse::<Ipv4Addr>()?;
    Ok(ipv4)
}

fn get_dns_records(
    client: &HttpApiClient,
    list_dns_endpoint: &ListDnsRecords<'_>,
) -> anyhow::Result<Vec<DnsRecord>> {
    let response = client.request(list_dns_endpoint)?;
    Ok(response.result)
}

fn get_patch_dns_endpoint<'a>(
    zone: &'a str,
    record: &'a DnsRecord,
    ipv4: Ipv4Addr,
) -> UpdateDnsRecord<'a> {
    UpdateDnsRecord {
        zone_identifier: zone,
        identifier: &record.id,
        params: UpdateDnsRecordParams {
            ttl: Some(record.ttl),
            proxied: Some(record.proxied),
            name: &record.name,
            content: DnsContent::A { content: ipv4 },
        },
    }
}
