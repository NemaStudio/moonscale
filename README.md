# moonscale - Easy ephemeral databases for previews
![Docker Image Version](https://img.shields.io/docker/v/nooverflow/moonscale)
![GitHub License](https://img.shields.io/github/license/nooverflow/moonscale)

Create on-demand, PlanetScale API compatible, ephemeral SQL databases, using a simple REST API. This is mainly targeted for short-lived databases used in project previews (eg: you get one database per pull-request)

## Setup 
Setting up this project requires a working Kubernetes cluster. Setting one up is outside of the scope of this document.

### Kube-janitor
Since this project creates ephemeral databases, it creates every kube resource with a TTL (Time-to-live). Kubernetes doesn't have any "cleanup" logic by default so we need to deploy a controller that will do it for us. 

A great open-source solution for this is [kube-janitor](https://codeberg.org/hjacobs/kube-janitor), you can easily deploy it using the Helm chart included in the repository.
```bash
git clone https://codeberg.org/hjacobs/kube-janitor.git
cd unsupported
  cat <<EOF
  cron:
    schedule: '*/1 * * * *'
  kind: CronJob
  kubejanitor:
    debug: true
    includeNamespaces:
    - moonscale
    includeResources:
    - statefulsets
    - secrets
    - configmaps
    - services
    - ingresses
    - persistentvolumeclaims
  EOF > values.yaml
helm install -n moonscale --create-namespace -f values.yaml kube-janitor ./helm 
```
