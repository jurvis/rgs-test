#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::copy;
    use bitcoin::blockdata::constants::genesis_block;
    use bitcoin::network::constants::Network;
    use lightning::util::logger::{Logger, Record};
    use lightning::routing::gossip::NetworkGraph;
    use lightning_rapid_gossip_sync::RapidGossipSync;
    use chrono::Utc;
    use lightning_rapid_gossip_sync::error::GraphSyncError;

    struct TestLogger {}
    impl Logger for TestLogger {
        fn log(&self, record: &Record) {
            let raw_log = record.args.to_string();
            let string = format!(
                "{} {:<5} [{}:{}] {}\n",
                // Note that a "real" lightning node almost certainly does *not* want subsecond
                // precision for message-receipt information as it makes log entries a target for
                // deanonymization attacks. For testing, however, its quite useful.
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level.to_string(),
                record.module_path,
                record.line,
                raw_log
            );
            println!("{}", string);
        }
    }

    #[tokio::test]
    async fn grab_rgs_graph() -> Result<(), Box<dyn std::error::Error>> {
        let logger = TestLogger{};

        let block_hash = genesis_block(Network::Testnet).header.block_hash();
        let network_graph = NetworkGraph::new(block_hash, &logger);
        let rapid_sync = RapidGossipSync::new(&network_graph);

        let url = "https://testnet-rgs.jurvis.co/snapshot/0/";
        let response = reqwest::get(url).await.unwrap();

        let filename = "./rapid_sync.lngossip";
        let mut dest = File::create(filename).unwrap();

        let content = response.text().await.unwrap();
        copy(&mut content.as_bytes(), &mut dest)?;

        match rapid_sync.sync_network_graph_with_file_path("./rapid_sync.lngossip") {
            Ok(snapshot) => {
                print!("Last sync snapshot: {}", snapshot);
                assert!(true)
            },
            Err(err) => {
                let message = match err {
                    GraphSyncError::DecodeError(_) => { "Ran into some decoding error" }
                    GraphSyncError::LightningError(_) => { "Ran into some Lighting error" }
                };
                assert!(false, "{}", message)
            }
        }

        Ok(())
    }
}
