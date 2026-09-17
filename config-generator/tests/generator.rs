use serde_json::json;
mod support;
use support::*;

fn user(id: u64) -> User {
	User {
		id,
		uuid: uuid::Uuid::new_v4().to_string(),
		password: uuid::Uuid::new_v4().simple().to_string(),
	}
}
fn ready() -> Result<State, String> {
	Ok(State {
		host: "tuic.example.com".into(),
		users: vec![user(0)],
		..State::initial()?
	})
}

#[test]
fn defaults_and_empty_state() -> Result<(), String> {
	let s = State::initial()?;
	assert!(validate(&s).contains_key("host"));
	assert!(build_configs(&s).is_err());
	assert_eq!(s.port, "8443");
	assert_eq!(s.listen, "[::]:8443");
	assert!(s.reconnect && s.lazy && !s.insecure && !s.zero_rtt);
	assert_eq!(Forward::initial()?.timeout, "60");
	Ok(())
}

#[test]
fn complete_default_pair() -> Result<(), String> {
	let s = ready()?;
	let u = &s.users[0];
	let expected = json!({
		"server": {"server":"[::]:8443", "log_level":"info", "users":{u.uuid.clone():u.password},
			"tls":{"hostname":"tuic.example.com", "alpn":["h3"], "certificate":"/etc/tuic/fullchain.pem", "private_key":"/etc/tuic/privatekey.pem"},
			"backend":{"mode":"quinn", "quinn":{"congestion_control":{"controller":"bbr", "initial_window":1048576},
				"initial_mtu":1200, "min_mtu":1200, "gso":true, "pmtu":true, "send_window":16777216, "receive_window":8388608, "max_idle_time":"30s"}},
			"auth_timeout":"3s", "stream_timeout":"60s",
			"outbound":{"default":{"type":"direct", "ip_mode":"v4first", "tfo":false}},
			"experimental":{"drop_loopback":true, "drop_private":true}},
		"client": {"server":"tuic.example.com:8443", "uuid":u.uuid, "password":u.password, "log_level":"info",
			"udp_relay_mode":"native", "heartbeat":"3s", "gc_interval":"3s", "gc_lifetime":"15s",
			"reconnect":true, "lazy":true,
			"reconnect_initial_backoff":"500ms", "reconnect_max_backoff":"30000ms",
			"tls":{"sni":"tuic.example.com", "alpn":["h3"], "skip_cert_verify":false},
			"backend":{"quinn":{"congestion_control":{"controller":"bbr"}, "send_window":16777216, "receive_window":8388608}},
			"local":{"server":"127.0.0.1:1080"}}
	});
	assert!(build_configs(&s)? == expected);
	Ok(())
}

#[test]
fn pairing_selection_and_removal() -> Result<(), String> {
	let mut s = ready()?;
	s.port = "443".into();
	s.users.push(user(1));
	s.active_user = 1;
	let config = build_configs(&s)?;
	assert!(config["server"]["users"][&s.users[1].uuid] == config["client"]["password"]);
	assert_eq!(config["server"]["server"], "[::]:8443");
	assert_eq!(config["client"]["server"], "tuic.example.com:443");
	let selected = s.users[1].uuid.clone();
	s.remove_user(0);
	assert_eq!(s.active_user, 0);
	assert!(s.users[0].uuid == selected);
	s.remove_user(1);
	assert_eq!(s.users.len(), 1);
	Ok(())
}

#[test]
fn invalid_users() -> Result<(), String> {
	let mut s = ready()?;
	let mut duplicate = s.users[0].clone();
	duplicate.uuid = duplicate.uuid.to_uppercase();
	s.users.push(duplicate);
	assert!(validate(&s).contains_key("users.1.uuid"));
	s.users[1].uuid = uuid::Uuid::nil().to_string();
	assert!(validate(&s).contains_key("users.1.uuid"));
	s.users[0].password.clear();
	assert!(validate(&s).contains_key("users.0.password"));
	s.active_user = 99;
	assert!(validate(&s).contains_key("activeUser"));
	s.users.clear();
	assert!(validate(&s).contains_key("users"));
	Ok(())
}

#[test]
fn independent_modes_ignore_inactive_values() -> Result<(), String> {
	let mut s = ready()?;
	s.mode = "server".into();
	s.host.clear();
	s.hostname = "tuic.example.com".into();
	s.local = "bad".into();
	assert!(build_configs(&s)?.get("client").is_none());
	s.mode = "client".into();
	s.host = "tuic.example.com".into();
	s.local = "127.0.0.1:1080".into();
	s.listen = "bad".into();
	s.tls_mode = "unsupported".into();
	s.controller = "unsupported".into();
	s.certificate.clear();
	s.private_key.clear();
	s.users.push(User::default());
	s.reconnect = false;
	s.max_backoff = "bad".into();
	assert!(build_configs(&s)?.get("server").is_none());
	Ok(())
}

#[test]
fn tls_conditions_and_explicit_verification() -> Result<(), String> {
	let mut s = ready()?;
	s.tls_mode = "self".into();
	assert!(validate(&s).contains_key("insecure"));
	s.insecure = true;
	let c = build_configs(&s)?;
	assert_eq!(c["server"]["tls"].as_object().map(|o| o.len()), Some(3));
	assert_eq!(c["server"]["tls"]["self_sign"], true);
	s.tls_mode = "acme".into();
	s.email = "admin@example.com".into();
	s.insecure = false;
	let c = build_configs(&s)?;
	assert_eq!(c["server"]["tls"]["auto_ssl"], true);
	assert!(c["server"]["tls"].get("certificate").is_none());
	s.tls_mode = "certificate".into();
	let c = build_configs(&s)?;
	assert!(c["server"].get("data_dir").is_none());
	assert!(c["server"]["tls"].get("auto_ssl").is_none());
	assert_eq!(c["client"]["tls"]["skip_cert_verify"], false);
	Ok(())
}

#[test]
fn ipv6_and_sni_fallbacks() -> Result<(), String> {
	let mut s = ready()?;
	s.host = "[2001:db8::1]".into();
	assert!(validate(&s).contains_key("sni"));
	s.hostname = "tuic.example.com".into();
	let c = build_configs(&s)?;
	assert_eq!(c["client"]["server"], "[2001:db8::1]:8443");
	assert_eq!(c["client"]["ip"], "2001:db8::1");
	assert_eq!(c["client"]["tls"]["sni"], "tuic.example.com");
	s.mode = "client".into();
	assert!(validate(&s).contains_key("sni"));
	s.sni = "tuic.example.com".into();
	assert!(validate(&s).is_empty());
	assert!(endpoint("::1:443", false).is_none());
	assert!(endpoint("[1.2.3.4]:443", false).is_none());
	Ok(())
}

#[test]
fn address_and_integer_boundaries() -> Result<(), String> {
	for host in [
		"0.0.0.0",
		"::",
		"https://example.com",
		"example.com/path",
		"host:443",
		"999.999.1.1",
		"a b.com",
		"01.2.3.4",
	] {
		let mut s = ready()?;
		s.host = host.into();
		assert!(validate(&s).contains_key("host"), "{host}");
	}
	for port in ["0", "65536", "1.2", "-1", "1e3", ""] {
		let mut s = ready()?;
		s.port = port.into();
		assert!(validate(&s).contains_key("port"));
	}
	for port in ["1", "65535"] {
		let mut s = ready()?;
		s.port = port.into();
		assert!(validate(&s).is_empty());
	}
	Ok(())
}

#[test]
fn reconnect_and_socks_limits() -> Result<(), String> {
	let mut s = ready()?;
	s.max_backoff = "100".into();
	assert!(validate(&s).contains_key("maxBackoff"));
	s.max_backoff = "9007199254740992".into();
	assert!(validate(&s).contains_key("maxBackoff"));
	s.reconnect = false;
	s.local_auth = true;
	s.local_username = "u".into();
	s.local_password = "字".repeat(86);
	assert!(validate(&s).contains_key("localPassword"));
	s.local_password = "字".repeat(85);
	assert!(validate(&s).is_empty());
	s.local_auth = false;
	let c = build_configs(&s)?;
	assert!(c["client"]["local"].get("password").is_none());
	assert!(c["client"].get("reconnect_initial_backoff").is_none());
	Ok(())
}

#[test]
fn forwarding_arrays_and_conflicts() -> Result<(), String> {
	let mut s = ready()?;
	s.forwards = vec![
		Forward {
			listen: "127.0.0.1:8080".into(),
			remote: "example.com:80".into(),
			timeout: "bad".into(),
			..Forward::initial()?
		},
		Forward {
			id: 1,
			protocol: "udp".into(),
			listen: "[::1]:8053".into(),
			remote: "[2001:db8::53]:53".into(),
			timeout: "045".into(),
		},
	];
	let c = build_configs(&s)?;
	assert!(c["client"]["local"]["tcp_forward"][0].get("timeout").is_none());
	assert_eq!(c["client"]["local"]["udp_forward"][0]["timeout"], "45s");
	s.forwards.push(s.forwards[1].clone());
	s.forwards[2].listen = "[0:0:0:0:0:0:0:1]:8053".into();
	assert!(validate(&s).contains_key("forwards.2.listen"));
	s.forwards[2].listen = "127.0.0.1:1080".into();
	assert!(validate(&s).contains_key("forwards.2.listen"));
	s.forwards[1].timeout = "0".into();
	assert!(validate(&s).contains_key("forwards.1.timeout"));
	Ok(())
}

#[test]
fn controllers_and_wired_client_transport() -> Result<(), String> {
	for (controller, _) in options("controller") {
		let mut s = ready()?;
		s.controller = controller.clone();
		let c = build_configs(&s)?;
		assert_eq!(
			c["server"]["backend"]["quinn"]["congestion_control"]["controller"],
			*controller
		);
		assert_eq!(c["client"]["udp_relay_mode"], "native");
		assert_eq!(c["client"]["backend"]["quinn"]["congestion_control"]["controller"], "bbr");
		assert!(c["client"].get("proxy").is_none());
		assert!(c["client"]["tls"].get("certificates").is_none());
	}
	Ok(())
}

#[test]
fn redaction_preserves_export() -> Result<(), String> {
	let mut s = ready()?;
	s.users.push(user(1));
	s.local_auth = true;
	s.local_username = "tester".into();
	s.local_password = " local secret ".into();
	let c = build_configs(&s)?;
	let before = c.clone();
	let preview = redact_configs(&c)?;
	for u in &s.users {
		assert_eq!(preview["server"]["users"][&u.uuid], "••••••••");
	}
	assert_eq!(preview["client"]["password"], "••••••••");
	assert_eq!(preview["client"]["local"]["password"], "••••••••");
	assert_eq!(preview["client"]["local"]["username"], "tester");
	assert!(c == before);
	Ok(())
}

#[test]
fn json5_line_separators_are_escaped() -> Result<(), String> {
	let mut s = ready()?;
	s.users[0].password = "\u{85}\u{2028}\u{2029}\"\\\n\0".into();
	let c = build_configs(&s)?;
	let text = serialize(&c["client"], "json")?;
	assert!(text.contains("\\u2028") && text.contains("\\u2029") && !text.contains('\u{2028}'));
	assert!(serde_json::from_str::<serde_json::Value>(&text).map_err(|_| "parse failed")? == c["client"]);
	assert!(serialize(&c, "xml").is_err());
	Ok(())
}

#[test]
fn form_dsl_binding_and_visibility() -> Result<(), String> {
	let mut s = State::initial()?;
	for f in input_fields() {
		assert!(f.write(&mut s, &f.read(&State::initial()?)));
	}
	assert!(s == State::initial()?);
	let fields = input_fields();
	let certificate = fields.iter().find(|f| f.key == "certificate");
	assert!(certificate.is_some_and(|f| f.visible(&s)));
	s.tls_mode = "acme".into();
	assert!(certificate.is_some_and(|f| !f.visible(&s)));
	Ok(())
}

fn outbound_row(id: u64, name: &str, kind: &str) -> serde_json::Value {
	json!({"id": id, "name": name, "type": kind, "ipMode": "v4first", "bindIpv4": "", "bindIpv6": "",
		"bindDevice": "", "tfo": false, "routingMarkEnabled": false, "routingMark": "",
		"socksAddr": "", "socksUsername": "", "socksPassword": "", "allowUdp": false})
}

#[test]
fn server_advanced_sections_cover_dev7() -> Result<(), String> {
	let mut s = ready()?;
	s.extra.insert("logEnabled".into(), json!(true));
	s.extra.insert("logFile".into(), json!("/var/log/tuic/server.log"));
	s.extra.insert("backendMode".into(), json!("quiche"));
	s.extra.insert("dnsEnabled".into(), json!(true));
	s.extra.insert("dnsMode".into(), json!("custom"));
	s.extra.insert(
		"dnsServers".into(),
		json!([{"id": 0, "server": "tls://1.1.1.1#cloudflare-dns.com"}]),
	);
	s.extra.insert("geodataEnabled".into(), json!(true));
	s.extra.insert("geosite".into(), json!("/var/lib/tuic/geosite.dat"));
	s.extra.insert("geoip".into(), json!("/var/lib/tuic/geoip.dat"));
	s.extra.insert("restfulEnabled".into(), json!(true));
	s.extra.insert("masqueradeEnabled".into(), json!(true));
	s.extra.insert(
		"acls".into(),
		json!([{"id": 0, "addr": "private", "outbound": "reject", "ports": "", "hijack": ""}]),
	);
	s.extra.insert("rules".into(), json!([{"id": 0, "rule": "MATCH,default"}]));
	s.extra.insert(
		"outbounds".into(),
		json!([outbound_row(0, "default", "direct"), outbound_row(1, "proxy", "socks5")]),
	);
	s.extra.get_mut("outbounds").expect("outbounds")[1]["socksAddr"] = json!("127.0.0.1:1080");
	assert!(validate(&s).is_empty());
	let c = build_configs(&s)?;
	assert_eq!(c["server"]["log"]["log_file"], "/var/log/tuic/server.log");
	assert_eq!(c["server"]["backend"]["mode"], "quiche");
	assert!(c["server"]["backend"].get("quinn").is_none());
	assert_eq!(c["server"]["backend"]["quiche"]["max_concurrent_bi_streams"], 100);
	assert_eq!(c["server"]["dns"]["servers"][0], "tls://1.1.1.1#cloudflare-dns.com");
	assert_eq!(c["server"]["geodata"]["geoip"], "/var/lib/tuic/geoip.dat");
	assert_eq!(c["server"]["restful"]["enabled"], true);
	assert_eq!(c["server"]["masquerade"]["enabled"], true);
	assert_eq!(c["server"]["acl"][0]["addr"], "private");
	assert_eq!(c["server"]["rules"][0], "MATCH,default");
	assert_eq!(c["server"]["outbound"]["default"]["type"], "direct");
	assert_eq!(c["server"]["outbound"]["proxy"]["addr"], "127.0.0.1:1080");
	assert!(c["server"]["outbound"]["proxy"].get("ip_mode").is_none());
	Ok(())
}

#[test]
fn client_transport_windows_and_0rtt() -> Result<(), String> {
	let mut s = ready()?;
	s.extra.insert("udpRelayMode".into(), json!("quic"));
	s.extra.insert("clientController".into(), json!("cubic"));
	s.extra.insert("clientSendWindow".into(), json!(4194304));
	s.extra.insert("clientReceiveWindow".into(), json!(2097152));
	s.zero_rtt = true;
	assert!(validate(&s).is_empty());
	let c = build_configs(&s)?;
	assert_eq!(c["client"]["udp_relay_mode"], "quic");
	assert_eq!(c["client"]["zero_rtt_handshake"], true);
	assert_eq!(c["client"]["backend"]["quinn"]["congestion_control"]["controller"], "cubic");
	assert_eq!(c["client"]["backend"]["quinn"]["send_window"], 4194304);
	assert_eq!(c["client"]["backend"]["quinn"]["receive_window"], 2097152);
	Ok(())
}
