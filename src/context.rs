pub struct Context {
    pub database_template_yaml_raw: String,
    pub kubernetes_client: kube::Client,
    pub ingress_domain: String,
}
