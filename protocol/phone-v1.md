# WestPlay phone event protocol v1

Transport: TLS WebSocket at /phone. mDNS service _westplay._tcp.local., TXT v=1, role=phone-data, pcId=<stablePCId>. Discovery only changes endpoint; pinned key must match saved PC.

Pair URI: westplay://phone?v=1&pc=<pcId>&host=<privateIPv4>&port=<1..65535>&fp=<64 lowercase hex SHA256 of certificate public-key SPKI DER>&invite=<64hex single-use secret>. Invite may be absent for an already-paired PC's endpoint update. First pairing requires invite. No plain HTTP / cleartext credential transport.

Client first message (<=4096 bytes):
- {"version":1,"type":"pair","invite":"...","deviceId":"random installed-app UUID","deviceName":"Pixel","platform":"android"}
- or {"version":1,"type":"authenticate","token":"...","deviceId":"...","deviceName":"Pixel","platform":"android"}.

Server success: {"version":1,"type":"ready","pcId":"...","token":"64hex token on first pair only, otherwise omitted","heartbeatMs":5000}. Store pcId/key pin/token encrypted by Android Keystore. Server stores only token SHA256. Errors: {"version":1,"type":"error","code":"unauthorized|alreadyPaired|expiredInvite|protocolMismatch|revoked|serviceStopped","message":"..."}. revoked clears stale credential; serviceStopped ends foreground link; transient I/O loss retries discovery with saved identity.

After ready every message is {"version":1,"sequence":positiveIncreasingInteger,"eventType":"...","payload":{...}}. Reset sequence to1 on a new connection. Server ack {"version":1,"type":"ack","sequence":n}; repeated/older sequence does not replay events. Send heartbeat every5s; PC declares loss after15s.

Events:
- deviceHello payload {"capabilities":{"battery":true,"charging":true,"notifications":true,"notificationBody":false,"backgroundRelay":true}}. Send first and whenever permissions/options change. Unsupported optional capabilities are ignored.
- deviceState payload {"batteryPct":0..100|null,"charging":true|false|null}. Send on connection + value changes.
- notificationPosted or notificationUpdated payload {"id":"stable sha256 of Android notification key","appId":"com.example","appName":"Example","title":"...","body":null|"..."}. Max id160, appId255, appName80, title160, body500 Unicode characters; total message8192 bytes. Body omitted/null unless user enabled sharing and capability notificationBody is true. Never send own foreground service, ongoing/media progress or secret/redacted notifications as recoverable private content.
- notificationRemoved payload {"id":"..."}.
- heartbeat and sessionEnded payload {}.

Do not send historical active notifications at first connection. Keep producer queue bounded; discard stale notification backlog on link loss. One paired device at a time. Device id is random, not hardware serial.

## Host-to-cabin types (camelCase)
PhoneState: {generation:number,connected:boolean,deviceId:string|null,deviceName:string|null,platform:'android'|'ios'|null,batteryPct:number|null,charging:boolean|null,capabilities:{battery:boolean,charging:boolean,notifications:boolean,notificationBody:boolean,backgroundRelay:boolean}}.
PhoneNotificationEvent: {kind:'posted'|'updated'|'removed',id:string,appId?:string,appName?:string,title?:string,body?:string|null}. Host receipt time controls UI freshness; no reliance on device wall clock.
HostMessage: {type:'phoneState',state:PhoneState} and {type:'phoneNotification',event:PhoneNotificationEvent}.

PhoneStatus (Tauri): {enabled:boolean,connected:boolean,pcId:string|null,address:string|null,port:number|null,pairUri:string|null,qrSvg:string|null,deviceName:string|null,platform:string|null,error:string|null,capabilities:Capabilities}.
phone_interfaces returns [{name:string,address:string}]. phone_status and phone_configure return PhoneStatus; phone_configure args {address:string|null}. phone_forget and phone_renew return PhoneStatus. New service has own user switch; existing mobile mirror is independent.

## Android user choices
Data link starts from visible user action; connectedDevice FGS keeps it alive while app screen off. Expose notification-access settings button, app filters, body-sharing default off, test event button, Stop and Forget actions. No automatic phone/Windows security setting changes. OS background restrictions must be shown honestly and tested, not silently ignored.
