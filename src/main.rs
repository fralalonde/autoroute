use structopt::StructOpt;

use serde::{Deserialize, Serialize};

use alsa::seq;
use alsa::seq::{Addr, PortSubscribe};

use std::collections::{HashSet, HashMap};
use std::fs::File;

#[derive(StructOpt, Debug)]
#[structopt(
name = "autoroute.py",
about = "Automatically connect USB MIDI devices to each other"
)]
enum Cmd {
    /// All active device
    Connect {
        config_file: String,
    },

    /// All known device
    Status,

    /// Install autoconnect trigger on USB event
    Install,
}

#[derive(Debug, PartialEq, Hash, Eq)]
struct Sub {
    src_client : i32,
    src_port: i32,
    dst_client: i32,
    dst_port: i32,
}


#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
enum PortDir {
    Duplex,
    Input,
    Output,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
enum Role {
    Broadcast,
    Monitor,
    Clock,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct ConfiguredDevice {
    port_name: String,
    #[serde(default = "default_port_dir")]
    port_dir: PortDir,
    alias: Option<String>,
    #[serde(default)]
    roles: Vec<Role>,
}

fn default_port_dir() -> PortDir {
    PortDir::Duplex
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct Config {
    devices: Vec<ConfiguredDevice>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct ConnectedDevice {
    config: Option<ConfiguredDevice>,
    client: i32,
    port: i32,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cmd = Cmd::from_args();

    match cmd {
        Cmd::Connect{config_file} => {
            let file = File::open(config_file)?;

            let config: Config = serde_yaml::from_reader(file).expect("loading config yaml");
            let dev_ports: HashMap<String, ConfiguredDevice> = config.devices.iter().map(| d | (d.port_name.clone(), d.clone())).collect();
            // let dev_alias: HashMap<String, ConfiguredDevice> = config.devices.iter().map(| d | (d.alias.clone(), d.clone())).collect();

            let seq = seq::Seq::open(None, None, false)?;

            let mut actual_subs: HashSet<Sub> = HashSet::new();
            let mut connected = vec![];

            let clients: Vec<_> = seq::ClientIter::new(&seq).collect();
            for client in &clients {
                if client.get_client() == 0 { continue }
                let ports: Vec<_> = seq::PortIter::new(&seq, client.get_client()).collect();
                for p in &ports {
                    if let Ok(name) = p.get_name() {
                        connected.push(ConnectedDevice { config:  dev_ports.get(name).cloned(), client: p.get_client(), port: p.get_port()});
                    }
                    // nameless devices ignored

                    let subs: Vec<seq::PortSubscribe> = seq::PortSubscribeIter::new(&seq, seq::Addr{client: p.get_client(), port: p.get_port()}, seq::QuerySubsType::WRITE).collect();
                    for s in &subs {
                        actual_subs.insert(Sub{src_client: s.get_sender().client, src_port: s.get_sender().port, dst_client: s.get_dest().client, dst_port: s.get_dest().port});
                    }
                }
            }

            let mut expected_subs: HashSet<Sub> = HashSet::new();
            for b in &connected {
                if let Some(config) = &b.config {
                    for t in &connected {
                        if t == b { continue }
                        if config.roles.contains(&Role::Broadcast) {
                            if match &t.config {
                                Some(config) => config.port_dir != PortDir::Input,
                                _ => true
                            } {
                                expected_subs.insert(Sub { src_client: b.client, src_port: b.port, dst_client: t.client, dst_port: t.port });
                            }
                        }
                        if config.roles.contains(&Role::Monitor) {
                            if let Some(tconfig) = &t.config {
                                if tconfig.roles.contains(&Role::Broadcast) {continue}
                            }
                            if match &t.config {
                                Some(config) => config.port_dir != PortDir::Output,
                                _ => true
                            } {
                                expected_subs.insert(Sub{src_client: t.client, src_port: t.port, dst_client: b.client, dst_port: b.port});
                            }
                        }
                    }
                }
            }

            for s in expected_subs.difference(&actual_subs) {
                let ps = PortSubscribe::empty()?;
                ps.set_sender(Addr{ client: s.src_client, port: s.src_port});
                ps.set_dest(Addr{ client: s.dst_client, port: s.dst_port});
                seq.subscribe_port(&ps)?
            }

            for s in actual_subs.difference(&expected_subs) {
                seq.unsubscribe_port(Addr{ client: s.src_client, port: s.src_port}, Addr{ client: s.dst_client, port: s.dst_port})?
            }
        }
        Cmd::Install => {
        }
        Cmd::Status => {
        }
    }
    Ok(())
}
