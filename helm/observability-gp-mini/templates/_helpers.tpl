{{/*
Expand the name of the chart.
*/}}
{{- define "observability-gp-mini.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Fully qualified app name, capped at 63 chars for the DNS label limit. This is
the chart-wide base; each component appends its own suffix (or takes its own
fullnameOverride) — see "observability-gp-mini.componentFullname" below.
*/}}
{{- define "observability-gp-mini.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{- define "observability-gp-mini.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Chart-wide labels, common to every component's resources.
*/}}
{{- define "observability-gp-mini.labels" -}}
helm.sh/chart: {{ include "observability-gp-mini.chart" . }}
{{ include "observability-gp-mini.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "observability-gp-mini.selectorLabels" -}}
app.kubernetes.io/name: {{ include "observability-gp-mini.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Per-component name/labels/selector-labels — one chart, several components
(otel-gateway, otel-node-collector, k8s-events-exporter, clickhouse), each
needing its own Service-selectable identity. Called with a dict, e.g.:
  {{ include "observability-gp-mini.componentFullname" (dict "ctx" $ "component" "otel-gateway" "override" .Values.otelGateway.fullnameOverride) }}
"component" becomes the app.kubernetes.io/component label and the default
name suffix; "override" is that component's own fullnameOverride value.
*/}}
{{- define "observability-gp-mini.componentFullname" -}}
{{- if .override }}
{{- .override | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" (include "observability-gp-mini.fullname" .ctx) .component | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "observability-gp-mini.componentSelectorLabels" -}}
{{ include "observability-gp-mini.selectorLabels" .ctx }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{- define "observability-gp-mini.componentLabels" -}}
{{ include "observability-gp-mini.labels" .ctx }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
A component's fullname, qualified with the release namespace — for any
resource that must stay unique across the whole cluster rather than just
within one namespace (a ClusterRole, say). otel-node-collector and
k8s-events-exporter both need this for their ClusterRole/ClusterRoleBinding;
their Deployment/DaemonSet/ServiceAccount use the plain componentFullname
above instead, since those are namespaced and get isolation from the
namespace itself.
*/}}
{{- define "observability-gp-mini.componentFullnameClusterUnique" -}}
{{- $fullname := include "observability-gp-mini.componentFullname" (dict "ctx" .ctx "component" .component "override" .override) }}
{{- printf "%s-%s" $fullname .ctx.Release.Namespace | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* ------------------------------------------------------------------ */}}
{{/* otel-gateway                                                        */}}
{{/* ------------------------------------------------------------------ */}}

{{- define "observability-gp-mini.otelGateway.fullname" -}}
{{- include "observability-gp-mini.componentFullname" (dict "ctx" . "component" "otel-gateway" "override" .Values.otelGateway.fullnameOverride) }}
{{- end }}

{{/* ------------------------------------------------------------------ */}}
{{/* otel-node-collector                                                 */}}
{{/* ------------------------------------------------------------------ */}}

{{- define "observability-gp-mini.otelNodeCollector.fullname" -}}
{{- include "observability-gp-mini.componentFullname" (dict "ctx" . "component" "otel-node-collector" "override" .Values.otelNodeCollector.fullnameOverride) }}
{{- end }}

{{- define "observability-gp-mini.otelNodeCollector.fullnameClusterUnique" -}}
{{- include "observability-gp-mini.componentFullnameClusterUnique" (dict "ctx" . "component" "otel-node-collector" "override" .Values.otelNodeCollector.fullnameOverride) }}
{{- end }}

{{- define "observability-gp-mini.otelNodeCollector.serviceAccountName" -}}
{{- if .Values.otelNodeCollector.serviceAccount.create }}
{{- default (include "observability-gp-mini.otelNodeCollector.fullname" .) .Values.otelNodeCollector.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.otelNodeCollector.serviceAccount.name }}
{{- end }}
{{- end }}

{{/* ------------------------------------------------------------------ */}}
{{/* k8s-events-exporter                                                 */}}
{{/* ------------------------------------------------------------------ */}}

{{- define "observability-gp-mini.k8sEventsExporter.fullname" -}}
{{- include "observability-gp-mini.componentFullname" (dict "ctx" . "component" "k8s-events-exporter" "override" .Values.k8sEventsExporter.fullnameOverride) }}
{{- end }}

{{- define "observability-gp-mini.k8sEventsExporter.fullnameClusterUnique" -}}
{{- include "observability-gp-mini.componentFullnameClusterUnique" (dict "ctx" . "component" "k8s-events-exporter" "override" .Values.k8sEventsExporter.fullnameOverride) }}
{{- end }}

{{- define "observability-gp-mini.k8sEventsExporter.serviceAccountName" -}}
{{- if .Values.k8sEventsExporter.serviceAccount.create }}
{{- default (include "observability-gp-mini.k8sEventsExporter.fullname" .) .Values.k8sEventsExporter.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.k8sEventsExporter.serviceAccount.name }}
{{- end }}
{{- end }}
