# Rotelle Cases


# Case status

## Implemented
### c0 idle

### c1 check: control response check mode

### c2 intermittent-01: k8s pod intermittent service failure
After N index accesses, application exits "crashes"

Input params:
- N acesses
Diagnoses:
- Failing service -> pod or pod -> service identified

### c3 intermittent-02: k8s pod intermittent service failure
k8s pod exceeds memory allocation over time

Input params:
- allocation loop time
- allocation loop amount

Diagnosis:
- identify root cause in ops env (the pod)
- propose workaround
- propose key fix

## Proposed
- Pod deployment failure in k8s config (missing config env)
- k8s public accessibility via k8s service (no external access path)
- k8s public accessibility via k8s service (overspecified loadbalancer and ingress controller both)
