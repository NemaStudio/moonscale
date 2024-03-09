// TODO: Refactor with a sub struct for config related variables
pub struct Context {
    pub database_template_yaml_raw: String,
    pub kubernetes_client: kube::Client,
    pub ingress_domain: String,
    pub resource_ttl: usize,
    pub api_key: String,
}
