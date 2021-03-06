extern crate uuid;
extern crate opus;
extern crate byteorder;

use self::uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use self::opus::{Decoder, Encoder, Application, Channels};
use self::byteorder::{ByteOrder, LittleEndian};

use sdp::{SessionDescription};
use rir::rtp::{RtpPkt, RtpHeader};
use convo::session_negotiation::{Session};

struct MemberSession {
    session: Session,
    r_payload: Mutex<Vec<u8>>,
    w_payload: Mutex<Vec<u8>>,
    init_packet: Mutex<Option<RtpPkt>>,
    counter: Mutex<u16>,
    ts: Mutex<u32>,
}

impl MemberSession {
    fn read_and_decode(&self, decoder: &mut opus::Decoder) -> Option<[u8; 3840]> {
        let mut buffer: [u8; 3840] = [0; 3840];

        let rtp_pkt = self.read_audio();
        let mut tmp_buffer: Vec<i16> = vec![0; 1920];

        debug!("Read from ssrc {} csrc {:?} seq {} ts {}...", rtp_pkt.header.ssrc, rtp_pkt.header.csrc, rtp_pkt.header.seq_number, rtp_pkt.header.timestamp);

        if rtp_pkt.payload.len() == 0 {
            return None
        }

        debug!("Before decoding1... {}", rtp_pkt.payload.len());
        let size = decoder.decode(&rtp_pkt.payload, &mut tmp_buffer, false).unwrap();
        debug!("After decoding... {}", size);
        debug!("After decoding2... {}", tmp_buffer.len());

        convert_i16_to_u8(&mut tmp_buffer, &mut buffer);

        let mut packet_lock = self.init_packet.lock().unwrap();
        if !packet_lock.is_some() {
            *packet_lock = Some(rtp_pkt);
        }

        return Some(buffer)
    }

    fn encode_and_write(&self, encoder: &mut opus::Encoder, raw_payload: [u8; 3840]) {
        let mut rtp_pkt;
        let packet_lock = self.init_packet.lock().unwrap();
        if packet_lock.is_some() {
            rtp_pkt = RtpPkt {
                header: RtpHeader {
                    version: 2,
                    padding: 0,
                    ext: 0,
                    cc: 0,
                    marker: 0,
                    payload_type: packet_lock.as_ref().unwrap().header.payload_type,
                    seq_number: packet_lock.as_ref().unwrap().header.seq_number + *(self.counter.lock().unwrap()),
                    timestamp: packet_lock.as_ref().unwrap().header.timestamp + *(self.ts.lock().unwrap()),
                    ssrc: packet_lock.as_ref().unwrap().header.ssrc,
                    csrc: packet_lock.as_ref().unwrap().header.csrc.clone(),
                },
                payload: vec![],
            };
            debug!("Writing ssrc {} csrc {:?} seq {} ts {}...", rtp_pkt.header.ssrc, rtp_pkt.header.csrc, rtp_pkt.header.seq_number, rtp_pkt.header.timestamp);
            *(self.ts.lock().unwrap()) += 960;
            *(self.counter.lock().unwrap()) += 1;
         } else {
            return;
        }

        let mut buffer = raw_payload;
        debug!("Buffer size {}", buffer.len());
        let mut tmp_buffer: Vec<i16> = vec![0; 1920];
        let mut encoded: [u8; 1920] = [0; 1920];

        convert_u8_to_i16(&mut buffer, &mut tmp_buffer);

        debug!("Before encoding1... {}", tmp_buffer.len());

        let size = encoder.encode(&tmp_buffer, &mut encoded).unwrap();

        debug!("After encoding1... {}", encoded.len());

        rtp_pkt.payload = vec![0; size];
        rtp_pkt.payload.clone_from_slice(&encoded[..size]);

        debug!("Writing packet with payload of size {}", rtp_pkt.payload.len());

        self.write_audio(&rtp_pkt);
    }

    fn write_audio(&self, rtp_pkt: &RtpPkt) {
        let sessions_map = self.session.media_sessions.read().unwrap();
        // TODO(tlam): Need to fix when we allocate more than 1 candidate, since this loop won't
        // work
        for (_, rtp_session) in sessions_map.iter() {
            rtp_session.write(rtp_pkt);
        }
    }

    fn read_audio(&self) -> RtpPkt {
        let mut rtp_pkt = RtpPkt {
            header: RtpHeader {
                version: 0,
                padding: 0,
                ext: 0,
                cc: 0,
                marker: 0,
                payload_type: 0,
                seq_number: 0,
                timestamp: 0,
                ssrc: 0,
                csrc: vec![],
            }, 
            payload: vec![],
        };

        let sessions_map = self.session.media_sessions.read().unwrap();

        for (_, rtp_session) in sessions_map.iter() {
            rtp_session.read(&mut rtp_pkt);
        }

        rtp_pkt
    }
}

#[derive(Copy, Clone)]
enum WorkerType {
    ReadWorker,
    WriteWorker,
}

struct MemberWorker {
    msess: Arc<MemberSession>,
    wtype: WorkerType,
}

impl MemberWorker {
    fn new(wtype: WorkerType, msess: Arc<MemberSession>) -> MemberWorker {
        MemberWorker {
            msess: msess,
            wtype: wtype,
        }
    }

    fn start_worker(self) {
        match self.wtype {
            WorkerType::ReadWorker => self.start_read_worker(),
            WorkerType::WriteWorker => self.start_write_worker(),
        }
    }

    fn start_read_worker(self) {
        thread::spawn(move || {
            let mut decoder = Decoder::new(48000, Channels::Stereo).unwrap();
            let mut count = 0;

            loop {
                // TODO(tlam): Read audio in session into the buffer
                if count < 100 {
                    thread::sleep(time::Duration::from_millis(5));
                    count += 1;
                    debug!("Audio worker waiting...");
                }

                debug!("Audio worker...1");
                let some_payload = self.msess.read_and_decode(&mut decoder);
                debug!("Audio worker...2");

                let mut payload = self.msess.r_payload.lock().unwrap();
                debug!("Audio worker...3");
                if some_payload.is_some() {
                    (*payload).extend_from_slice(&some_payload.unwrap());
                    //debug!("Audio worker payload... {:?}", (*payload));
                }
            }
        });
    }

    fn start_write_worker(self) {
        thread::spawn(move || {
            let mut encoder = Encoder::new(48000, Channels::Stereo, Application::Audio).unwrap();

            loop {
                // TODO(tlam): Write audio passed into the buffer into session
                thread::sleep(time::Duration::from_millis(10));

                let mut payload = self.msess.w_payload.lock().unwrap();
                if (*payload).len() < 3840 {
                    continue;
                }

                //debug!("Write worker payload...{:?}", (*payload));

                let mut slice = [0; 3840];
                let first = (*payload).split_off(3840);
                slice.clone_from_slice(&(*payload));
                *payload = first;
                self.msess.encode_and_write(&mut encoder, slice);
            }
        });
    }
}

pub struct Member {
    pub id: String,
    pub sdp: SessionDescription,
    member_session: Arc<MemberSession>,
}

impl Member {
    pub fn new(sdp: SessionDescription) -> Member {
        let member_id: &str = &Uuid::new_v4().to_string();

        debug!("Creating a new member [{}]", member_id);

        let session = Session::new(sdp.clone());

        let member = Member {
            id: member_id.to_string(),
            sdp: sdp,
            member_session: Arc::new(MemberSession {
                session: session,
                r_payload: Mutex::new(Vec::new()),
                w_payload: Mutex::new(Vec::new()),
                init_packet: Mutex::new(None),
                counter: Mutex::new(0),
                ts: Mutex::new(0),
            }),
        };

        member
    }

    pub fn get_read_payload(&self) -> Option<[u8; 3840]> {
        let member_session = self.member_session.clone();

        let mut x: [u8; 3840] = [0; 3840];

        let mut payload = member_session.r_payload.lock().unwrap();
        if payload.len() < 3840 {
            return None
        }

        let first = (*payload).split_off(3840);
        x.clone_from_slice(&(*payload));
        /*  Start dropping data after 1 sec */
        if first.len() >= 192000 {
            debug!("Deleting payload...");
            *payload = Vec::new();
        } else {
            *payload = first;
        }

        Some(x)
    }

    pub fn set_write_payload(&self, payload: [u8; 3840]) {
        let member_session = self.member_session.clone();

        let mut w_payload = member_session.w_payload.lock().unwrap();
        (*w_payload).extend_from_slice(&payload);
    }

    pub fn init_session(&self) {
        //self.session.init(Box::new(self.set_default_session));

        // TODO(tlam): Remove logic from init function
        self.member_session.session.process_offer();

        let rworker = MemberWorker::new(WorkerType::ReadWorker,
                                        self.member_session.clone());
        rworker.start_worker();
        let wworker = MemberWorker::new(WorkerType::WriteWorker,
                                        self.member_session.clone());
        wworker.start_worker();
    }

    pub fn negotiate_session(&self, base_sdp: Option<SessionDescription>) {
        // Pass base SDP and negotiate with session's offer
        let sdp_answer = self.member_session.session.negotiate_with_base_sdp(base_sdp);

        // Now that we have the answer we can process it
        self.member_session.session.process_answer();

        sdp_answer
    }

    pub fn get_session_answer(&self) -> SessionDescription {
        self.member_session.session.answer_sdp.read().unwrap().clone().unwrap()
    }
}

// Utils

fn convert_u8_to_i16(orig: &mut [u8], dest: &mut [i16]) {

    for i in 0..dest.len() {
        dest[i] = LittleEndian::read_i16(&[orig[i*2], orig[(i*2)+1]]);
    }
}

fn convert_i16_to_u8(orig: &mut [i16], dest: &mut [u8]) {
    for i in 0..orig.len() {
        //dest[i] = orig[i*2] as u16;
        //dest[i] = dest[i] << 8;

        dest[i*2] = (orig[i] & 0xFF) as u8;
        dest[i*2 + 1] = ((orig[i] >> 8) & 0xFF) as u8;
        //dest[i] = dest[i] | (orig[i*2 +1] as u8);
    }
}
