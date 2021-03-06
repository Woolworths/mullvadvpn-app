mod resolvconf;
mod static_resolv_conf;

use std::env;
use std::net::IpAddr;

use self::resolvconf::Resolvconf;
use self::static_resolv_conf::StaticResolvConf;

error_chain! {
    errors {
        NoDnsSettingsManager {
            description("No DNS settings manager detected")
        }
    }

    links {
        Resolvconf(resolvconf::Error, resolvconf::ErrorKind);
        StaticResolvConf(static_resolv_conf::Error, static_resolv_conf::ErrorKind);
    }
}

pub enum DnsSettings {
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
}

impl DnsSettings {
    pub fn new() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        Ok(match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsSettings::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsSettings::Resolvconf(Resolvconf::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        })
    }

    fn with_detected_dns_manager() -> Result<Self> {
        Resolvconf::new()
            .map(DnsSettings::Resolvconf)
            .or_else(|_| StaticResolvConf::new().map(DnsSettings::StaticResolvConf))
            .chain_err(|| ErrorKind::NoDnsSettingsManager)
    }

    pub fn set_dns(&mut self, interface: &str, servers: Vec<IpAddr>) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.set_dns(servers)?,
        }

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        use self::DnsSettings::*;

        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset()?,
        }

        Ok(())
    }
}
