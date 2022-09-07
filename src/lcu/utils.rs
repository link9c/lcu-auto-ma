use std::collections::HashMap;

pub fn parse_cmd_outoput(r: &str) -> HashMap<String, String> {
    // let r = "agueClient/LeagueClientUx.exe \"--riotclient-auth-token=psm0ORo2keEAcoOaMcw8ww\" \"--riotclient-app-port=63316\" \"--riotclient-tencent\" \"--no-rads\" \"--disable-self-update\" \"--region=TENCENT\" \"--locale=zh_CN\" \"--t.lcdshost=hn1-sz-feapp.lol.qq.com\" \"--t.chathost=hn1-sz-ejabberd.lol.qq.com\" \"--t.lq=https://hn1-sz-login.lol.qq.com:8443\" \"--t.storeurl=https://hn1-sr.lol.qq.com:8443\" \"--t.rmsurl=wss://sz-rms-bcs.lol.qq.com:443\" \"--rso-auth.url=https://prod-rso.lol.qq.com:3000\" \"--rso_platform_id=HN1\" \"--rso-auth.client=lol\" \"--t.location=loltencent.sz.HN1\" \"--tglog-endpoint=https://tglogsz.datamore.qq.com/lolcli/report/\" \"--t.league_edge_url=https://ledge-hn1.lol.qq.com:22019\" \"--ccs=https://cc-hn1.lol.qq.com:8093\" \"--dradis-endpoint=http://some.url\" \"--remoting-auth-token=tzwyGAFfukloNZyEgUR02w\" \"--app-port=63481\" \"--install-directory=e:\\game\\英雄联盟\\LeagueClient\" \"--app-name=LeagueClient\" \"--ux-name=LeagueClientUx\" \"--ux-helper-name=LeagueClientUxHelper\" \"--log-dir=LeagueClient Logs\" \"--crash-reporting=\" \"--crash-environment=HN1\" \"--app-log-file-path=e:/game/Ӣ������/LeagueClient/../Game/Logs/LeagueClient Logs/2022-09-04T14-52-04_20576_LeagueClient.log\" \"--app-pid=20576\" \"--output-base-dir=e:/game/英雄联盟/LeagueClient/../Game\" \"--no-proxy-server\"  \r\r\n\r\r\n";
    let mut result: HashMap<String, String> = HashMap::new();
    for args in r.replace('\"', "").replace(' ', "").split("--") {
        let arg_vec = args.split('=').collect::<Vec<&str>>();

        if arg_vec[0] == "app-port" || arg_vec[0] == "remoting-auth-token" {
            result.insert(arg_vec[0].to_string(), arg_vec[1].to_string());
        }
    }
    result
}

