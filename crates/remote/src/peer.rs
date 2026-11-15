use crate::{Input, control::Run};
use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use webrtc::{
    api::{
        APIBuilder,
        interceptor_registry::register_default_interceptors,
        media_engine::{MIME_TYPE_H264, MediaEngine},
        setting_engine::SettingEngine,
    },
    data_channel::data_channel_message::DataChannelMessage,
    ice::network_type::NetworkType,
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::{
        RTCPFeedback,
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters},
    },
    track::track_local::{TrackLocal, track_local_static_sample::TrackLocalStaticSample},
};

pub async fn answer(state: Arc<Run>, offer: RTCSessionDescription) -> Result<RTCSessionDescription, String> {
    // Browser mDNS host candidates are LAN-scoped. Reject public IP candidates,
    // TCP candidates and relay/srflx candidates, even in authenticated requests.
    for line in offer.sdp.lines().filter(|l| l.starts_with("a=candidate:")) {
        let fields: Vec<_> = line.split_whitespace().collect();
        let ip = fields.get(4).copied().unwrap_or_default();
        if fields.get(2).is_none_or(|v| !v.eq_ignore_ascii_case("udp"))
            || fields.get(7) != Some(&"host")
            || !(ip.ends_with(".local") || ip.parse::<std::net::Ipv4Addr>().is_ok_and(crate::allowed_address))
        {
            return Err("Only local IPv4 UDP host candidates are supported.".into());
        }
    }
    let mut media = MediaEngine::default();
    media
        .register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_H264.into(),
                    clock_rate: 90_000,
                    sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f".into(),
                    rtcp_feedback: vec![RTCPFeedback { typ: "nack".into(), parameter: "pli".into() }],
                    ..Default::default()
                },
                payload_type: 102,
                ..Default::default()
            },
            webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video,
        )
        .map_err(|e| e.to_string())?;
    let registry = register_default_interceptors(Registry::new(), &mut media).map_err(|e| e.to_string())?;
    let mut settings = SettingEngine::default();
    settings.set_network_types(vec![NetworkType::Udp4]);
    let bind = state.bind;
    settings.set_ip_filter(Box::new(move |ip| ip == std::net::IpAddr::V4(bind)));
    let api = APIBuilder::new()
        .with_media_engine(media)
        .with_interceptor_registry(registry)
        .with_setting_engine(settings)
        .build();
    let pc = Arc::new(api.new_peer_connection(RTCConfiguration::default()).await.map_err(|e| e.to_string())?);
    let track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.into(),
            clock_rate: 90_000,
            sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f".into(),
            ..Default::default()
        },
        "screen".into(),
        "westplay".into(),
    ));
    let sender = pc.add_track(track.clone() as Arc<dyn TrackLocal + Send + Sync>).await.map_err(|e| e.to_string())?;
    tokio::spawn(async move { while sender.read_rtcp().await.is_ok() {} });
    let s = state.clone();
    pc.on_data_channel(Box::new(move |channel| {
        let s = s.clone();
        Box::pin(async move {
            let overlay = channel.label() == "map-overlay";
            if (overlay && (channel.ordered() || channel.max_packet_lifetime().is_none_or(|ms| ms == 0 || ms > 150)))
                || (!overlay && channel.label() != "control")
            {
                let _ = channel.close().await;
                return;
            }
            if overlay {
                let overlay_state = s.clone();
                let overlay_channel = Arc::downgrade(&channel);
                tokio::spawn(async move {
                    let start = Instant::now();
                    let mut ever_connected = false;
                    let mut last_packet = String::new();
                    loop {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        let Some(channel) = overlay_channel.upgrade() else { break };
                        use webrtc::data_channel::data_channel_state::RTCDataChannelState;
                        if overlay_state.stop.load(Ordering::Acquire)
                            || matches!(
                                channel.ready_state(),
                                RTCDataChannelState::Closed | RTCDataChannelState::Closing
                            )
                        {
                            break;
                        }
                        let connected = overlay_state.connected.load(Ordering::Acquire);
                        if !connected {
                            if ever_connected || start.elapsed() > Duration::from_secs(15) { break } else { continue }
                        }
                        ever_connected = true;
                        if channel.ready_state() != RTCDataChannelState::Open || channel.buffered_amount().await > 16384
                        {
                            continue;
                        }
                        let now = Instant::now();
                        let key = overlay_state.observer.projection_source(now).map(|(key, _)| key);
                        let packet = {
                            let projection = overlay_state.projection.lock().unwrap();
                            serde_json::json!({"type":"mapOverlay","projection":projection.get(key,now)}).to_string()
                        };
                        if packet == last_packet {
                            continue;
                        }
                        last_packet = packet.clone();
                        if !matches!(
                            tokio::time::timeout(Duration::from_millis(150), channel.send_text(packet)).await,
                            Ok(Ok(_))
                        ) {
                            break;
                        }
                    }
                });
                return;
            }
            let weak = Arc::downgrade(&channel);
            channel.on_message(Box::new(move |message: DataChannelMessage| {
                let s = s.clone();
                let channel = weak.clone();
                Box::pin(async move {
                    if message.data.len() > 4096 || !message.is_string || s.stop.load(Ordering::Acquire) {
                        return;
                    }
                    if message.data.as_ref() == b"ping" {
                        *s.heartbeat.lock().unwrap() = Instant::now();
                        if let Some(c) = channel.upgrade() {
                            let _ = c.send_text("pong").await;
                        }
                        return;
                    }
                    if let Ok(input) = serde_json::from_slice::<Input>(&message.data)
                        && input.valid()
                    {
                        *s.heartbeat.lock().unwrap() = Instant::now();
                        s.inputs.lock().unwrap().push(input);
                    }
                })
            }));
        })
    }));
    let s = state.clone();
    pc.on_peer_connection_state_change(Box::new(move |status| {
        let s = s.clone();
        Box::pin(async move {
            if status == RTCPeerConnectionState::Connected {
                *s.heartbeat.lock().unwrap() = Instant::now();
                s.connected.store(true, Ordering::Release);
            } else if matches!(
                status,
                RTCPeerConnectionState::Disconnected | RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
            ) {
                s.release();
            }
        })
    }));
    let result: Result<RTCSessionDescription, String> = async {
        pc.set_remote_description(offer).await.map_err(|e| e.to_string())?;
        let answer = pc.create_answer(None).await.map_err(|e| e.to_string())?;
        let mut done = pc.gathering_complete_promise().await;
        pc.set_local_description(answer).await.map_err(|e| e.to_string())?;
        tokio::time::timeout(Duration::from_secs(8), done.recv())
            .await
            .map_err(|_| "Local ICE gathering timed out".to_owned())?;
        pc.local_description().await.ok_or("Missing local description".into())
    }
    .await;
    if result.is_err() {
        let _ = pc.close().await;
        return result;
    }
    *state.peer.lock().await = Some(pc.clone());
    tokio::spawn(async move {
        let start = Instant::now();
        while !state.connected.load(Ordering::Acquire)
            && start.elapsed() < Duration::from_secs(15)
            && !state.stop.load(Ordering::Acquire)
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        if state.connected.load(Ordering::Acquire) && !state.stop.load(Ordering::Acquire) {
            match crate::encoder::Encoder::start(state.clone()) {
                Ok((_encoder, mut frames)) => {
                    let mut timer = tokio::time::interval(Duration::from_millis(100));
                    loop {
                        tokio::select! {
                            frame = frames.recv() => {
                                let Some(data) = frame else { state.fail("Hardware video encoder stopped.".into()); break; };
                                if track.write_sample(&Sample { data: data.into(), duration: Duration::from_micros(33_333), ..Default::default() }).await.is_err() { break; }
                                state.frames.fetch_add(1, Ordering::Relaxed);
                            }
                            _ = timer.tick() => {
                                if state.stop.load(Ordering::Acquire) || !state.connected.load(Ordering::Acquire)
                                    || state.heartbeat.lock().unwrap().elapsed() > Duration::from_secs(3) { break; }
                            }
                        }
                    }
                }
                Err(e) => state.fail(e),
            }
        }
        state.release();
        let _ = pc.close().await;
        state.peer.lock().await.take();
        state.negotiating.store(false, Ordering::Release);
    });
    result
}
