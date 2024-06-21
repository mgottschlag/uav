use bevy::log::{error, info};
use nrf24l01_stick_driver::{
    Configuration, CrcMode, DataRate, Receiver, MAX_PAYLOAD_LEN, NRF24L01,
};
use protocol::{Command, Status};
use std::path::PathBuf;

pub struct Radio {
    receiver: Receiver,
    cmd_queue: tokio::sync::mpsc::Receiver<Command>,
    last_cmd: Command,
    status_queue: tokio::sync::mpsc::Sender<Status>,
}

impl Radio {
    pub async fn new(
        device: PathBuf,
        cmd_queue: tokio::sync::mpsc::Receiver<Command>,
        status_queue: tokio::sync::mpsc::Sender<Status>,
    ) -> Self {
        let mut config = Configuration::default();
        config.channel = 0x32;
        config.rate = DataRate::R2Mbps;
        config.power = 3;
        config.crc = Some(CrcMode::OneByte);
        config.auto_retransmit_delay_count = Some((250, 3));

        let mut nrf24l01 = NRF24L01::open_default(config, device.to_str().unwrap())
            .await
            .expect("could not open device");
        // data is received via ACK payloads -> no need to set any receive addresses
        nrf24l01
            .set_receive_addr(None, None, None, None, None)
            .await
            .expect("could not set receive address");

        let receiver = nrf24l01.receive().await.expect("could not start receiving");

        Radio {
            receiver: receiver,
            cmd_queue: cmd_queue,
            last_cmd: Command::new(),
            status_queue: status_queue,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                packet = self.receiver.receive() => {
                    let packet = packet.expect("could not receive packet");
                    info!("Received {:?} from {}\r", packet.payload, packet.pipe);
                }
                Some(cmd) = self.cmd_queue.recv() => {
                    if cmd != self.last_cmd {
                        self.send(cmd.clone()).await;
                        self.last_cmd = cmd;
                    }

                },
            }
        }
    }

    pub async fn send(&mut self, mut cmd: Command) {
        info!("Sending: {cmd:?}");
        cmd.thrust = cmd.thrust.map(|e| e.clamp(0, 255));
        cmd.pose = cmd.pose.map(|e| e.clamp(-1.0, 1.0));

        let size = minicbor::len(&cmd); // TODO: handle size >= MAX_PAYLOAD_LEN bytes
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        minicbor::encode(&cmd, buf.as_mut()).unwrap();

        match self
            .receiver
            .send(
                (&[0x44u8, 0x72u8, 0x6fu8, 0x6eu8, 0x65u8][..]).into(),
                &buf[..size],
            )
            .await
        {
            Ok(Some(ack_payload)) => {
                let data = ack_payload.payload;
                let _size = data.len();
                //info!("Received ACK payload: {data:?}. len={size}\r");

                let status: Status = minicbor::decode(&data[..]).unwrap();
                /*info!(
                    "roll={}, pitch={}, battery={}\r",
                    status.roll, status.pitch, status.battery
                );*/
                self.status_queue.send(status).await.unwrap();
            }
            Ok(None) => {
                info!("Did not receive ACK payload.\r");
            }
            Err(e) => error!("could not send: {e:?}\r"),
        }
    }
}
