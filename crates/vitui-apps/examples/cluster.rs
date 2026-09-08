//! **`cluster` — k9s, drawn on the component surface.**
//!
//! ```text
//! cargo run -p vitui-apps --example cluster
//! ```
//!
//! <https://github.com/derailed/k9s>
//!
//! # A port, and the cluster is the only invented part
//!
//! The shape is k9s's: the cluster-info block, the namespace and action mnemonics, the ASCII logo,
//! one resource table under a `Pods(default)[9]` title, and the breadcrumb trail along the bottom.
//! The logo is `internal/ui/splash.go`'s `LogoSmall` verbatim, the cluster-info rows are
//! `internal/view/cluster_info.go`'s seven, and the pod columns are `internal/render/pod.go`'s, in
//! its order, narrowed to the ones that are not `Wide` — `Ctrl+W` brings two of those back.
//!
//! **There is no cluster and no `kubectl`.** Nine pods, four deployments, four services, three
//! nodes and five namespaces are hashed out of their own names, so two runs agree and a *refresh*
//! is a deliberate change rather than a different random draw. A real client would put the
//! interesting failures in the transport instead of in the library under test.
//!
//! # What a resource view is here: a stack, and the breadcrumbs are that stack
//!
//! k9s is a stack of views and `Esc` is *pop*. So is this: `Enter` on a pod pushes its containers,
//! `l` pushes its logs, `y` its YAML and `d` its description, and the crumb row is
//! `App::stack` drawn with one [`chip_with`] a level. A text view — logs, YAML, describe — is a level
//! like any other, which is why it scrolls with the same keys the table does and leaves with the
//! same `Esc`.
//!
//! # Three things the port cannot say, recorded rather than worked around
//!
//! 1. **A table header is drawn in one role and one alignment.** k9s right-aligns `RESTARTS`,
//!    `CPU` and `MEM` in both the header and the cells; `Column` carries a title and a width and no
//!    justification, so the cells here are right-aligned by their drawer and the headers are not.
//! 2. **`PanelOpts` has no title alignment**, so `Pods(default)[9]` sits at the left where k9s
//!    centres it. `counter` recorded this first; `commander` met it again; this is the third.
//! 3. **A command prompt cannot live in the border.** k9s slides its `:` prompt over the header;
//!    here it is a panel of its own that replaces the header's rows, because an overlay would put
//!    a scrim over a screen that is not modal.
//!
//! # Three keys k9s binds that a `collection` ate, and the one reading behind all three
//!
//! `crate::collect::from_key` answers a bare `Space` with `Gesture::Toggle` and `Ctrl+A` with
//! `Gesture::All` in **every** [`Mode`], and `apply` then ignores both at [`Mode::Cursor`] — so the
//! key was consumed to do nothing and never reached this application. `crate::nav::step` read `←`
//! and `→` as `↑` and `↓`, which sent those two the same way for a different reason.
//!
//! **It was `owns_escape`'s defect one key over.** `Escape` had been taught to
//! decline when `apply` would clear nothing — *the component owns it exactly when it would clear
//! something* — and left `Space` and `Ctrl+A` as they were, so k9s's `space` was `Ctrl+Space` here
//! for the life of this port. `crate::collect::owns` is that narrowing said of the whole
//! vocabulary and the bare key arrives now; the chord stays bound beside it.
//!
//! **`←` and `→` are not the collection's any more**, and that one was never about ownership: a
//! group declares its axis, `nav::step` reads it, and a vertical list declines the two arrows it
//! does not have. This port wanted them least of the three — k9s walks its levels with `Esc` and a
//! colon command — and `commander` and `spf` bind what their originals bind because of it.
//!
//! # The letters are commands, so the table's type-ahead must never match
//!
//! `d`, `l`, `y`, `s` and the rest are k9s's verbs, and **a focused collection eats every
//! text-bearing key into its type-ahead buffer**. The same reading `commander` uses works
//! here: the caller's search answers `None`, the component declines the key, and it arrives in
//! `Driver::unhandled`. Filtering is a *field* — `/` — which is where a search belongs and what
//! the collection's own budget note says out loud.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | `:` | the command prompt — `pods`, `deploy`, `svc`, `no`, `ns`, `q` |
//! | `/` | the filter, over the rows on screen |
//! | `Enter` | drill in: a pod's containers, a deployment's pods, a namespace's pods |
//! | `Esc` | pop one level; from the root, clear the filter |
//! | `d` `y` `l` | describe · YAML · logs, each a level of its own |
//! | `space` · `Ctrl+Space` | mark — the bare key is k9s's own and arrives now, the chord stays bound beside it (see below) · `Ctrl+\` clears |
//! | `Ctrl+D` | delete what is marked, or the row under the cursor. **`Tab` or `←`/`→`, then
//!   `Enter`** — the dialog opens on `Cancel`, which is k9s's own *requires TAB and ENTER
//!   confirmation* |
//! | `Shift+N` `Shift+A` `Shift+S` | sort by name · age · status |
//! | `0` … `5` | the namespace mnemonics down the middle of the header |
//! | `Ctrl+W` `Ctrl+E` `Ctrl+G` | wide columns · the header · the breadcrumbs |
//! | `Ctrl+R` | refresh: new metrics, one minute older |
//! | `?` | help · `Ctrl+Q` or `:q` quits |

use std::fmt::Write as _;

use vitui_components::collect::{
    Cell, CollOpts, Column, Gesture, Mode, Selection, TableOpts, TableState, apply, table,
};
use vitui_components::edit::Text;
use vitui_components::frame::{Face, face_role};
use vitui_components::indicate::{MeterOpts, meter_with};
use vitui_components::input::{ButtonOpts, button_with, field};
use vitui_components::order::Rows;
use vitui_components::overlay::{Kind as ShellKind, ShellOpts, overlay_with};
use vitui_components::scroll::{Span, scrollbar};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{ChipOpts, FitOpts, Justify, chip_with, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, Code, KeyMap, Pressed};
use vitui_runtime::layout::Constraint::{Fixed, Weight};
use vitui_runtime::layout::{Col, Row, Stack, rect};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Interest, OverlayOpts, Rect, Revision, Role, Themes, Z};

// ── the cluster, which is the only invented thing here ───────────────────────────────────────────

/// A deterministic 64-bit hash. FNV-1a; two runs agree, which is the whole requirement.
fn hash(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A pod phase, and the role it is drawn in.
///
/// k9s colours these and so does this — through a `Role`, never a colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Running,
    Pending,
    ContainerCreating,
    CrashLoopBackOff,
    ImagePullBackOff,
    Terminating,
    Completed,
    Error,
}

impl Phase {
    const fn word(self) -> &'static str {
        match self {
            Phase::Running => "Running",
            Phase::Pending => "Pending",
            Phase::ContainerCreating => "ContainerCreating",
            Phase::CrashLoopBackOff => "CrashLoopBackOff",
            Phase::ImagePullBackOff => "ImagePullBackOff",
            Phase::Terminating => "Terminating",
            Phase::Completed => "Completed",
            Phase::Error => "Error",
        }
    }

    /// What a reader is meant to feel about it.
    const fn role(self) -> Role {
        match self {
            Phase::Running => Role::Ok,
            Phase::Pending | Phase::ContainerCreating | Phase::Terminating => Role::Warn,
            Phase::CrashLoopBackOff | Phase::ImagePullBackOff | Phase::Error => Role::Danger,
            Phase::Completed => Role::Dim,
        }
    }

    /// Whether a pod in this phase is counted as up.
    const fn healthy(self) -> bool {
        matches!(self, Phase::Running | Phase::Completed)
    }
}

/// One container of one pod.
struct Container {
    name: String,
    image: String,
    ready: bool,
    state: Phase,
    restarts: u32,
    cpu: u32,
    mem: u32,
}

/// One pod.
struct Pod {
    ns: usize,
    name: String,
    owner: usize,
    phase: Phase,
    restarts: u32,
    ip: String,
    node: usize,
    /// Minutes since it was created. The `AGE` column, and what `Ctrl+R` moves.
    age: u32,
    qos: &'static str,
    containers: Vec<Container>,
}

impl Pod {
    /// `2/2`, and whether both halves agree.
    fn ready(&self) -> (usize, usize) {
        (
            self.containers.iter().filter(|c| c.ready).count(),
            self.containers.len(),
        )
    }

    /// Millicores and mebibytes, summed over the containers — which is where k9s gets them.
    fn usage(&self) -> (u32, u32) {
        self.containers
            .iter()
            .fold((0, 0), |(c, m), k| (c + k.cpu, m + k.mem))
    }
}

/// One deployment.
struct Deployment {
    ns: usize,
    name: String,
    want: u32,
    age: u32,
}

/// One service.
struct Service {
    ns: usize,
    name: String,
    kind: &'static str,
    cluster_ip: String,
    external: String,
    ports: String,
    age: u32,
}

/// One node.
struct Node {
    name: String,
    role: &'static str,
    version: &'static str,
    ready: bool,
    /// Millicores and mebibytes it can hold, which is the denominator of the header's two meters.
    capacity: (u32, u32),
    age: u32,
}

/// Everything the invented cluster holds.
struct Cluster {
    namespaces: Vec<&'static str>,
    nodes: Vec<Node>,
    deployments: Vec<Deployment>,
    services: Vec<Service>,
    pods: Vec<Pod>,
}

/// The five namespaces, and the mnemonics `0`..`4` switch between them. `<0>` is *all*.
const NAMESPACES: [&str; 5] = [
    "default",
    "kube-system",
    "monitoring",
    "ingress",
    "flux-system",
];

/// What a deployment is made of: a name, a namespace, a replica count and an image.
const WORKLOADS: [(&str, usize, u32, &str); 8] = [
    ("api-gateway", 0, 3, "ghcr.io/acme/gateway:1.14.2"),
    ("checkout", 0, 2, "ghcr.io/acme/checkout:2.7.0"),
    ("search-indexer", 0, 1, "ghcr.io/acme/indexer:0.9.4"),
    ("coredns", 1, 2, "registry.k8s.io/coredns/coredns:v1.11.1"),
    (
        "metrics-server",
        1,
        1,
        "registry.k8s.io/metrics-server:v0.7.1",
    ),
    ("prometheus", 2, 1, "quay.io/prometheus/prometheus:v2.53.0"),
    (
        "ingress-nginx",
        3,
        2,
        "registry.k8s.io/ingress-nginx:v1.11.2",
    ),
    (
        "flux-controller",
        4,
        1,
        "ghcr.io/fluxcd/source-controller:v1.3.0",
    ),
];

impl Cluster {
    /// The cluster this application opens on.
    fn seed() -> Cluster {
        let nodes = vec![
            Node {
                name: "kind-vitui-control-plane".to_owned(),
                role: "control-plane",
                version: "v1.30.2",
                ready: true,
                capacity: (4000, 8192),
                age: 61 * 1440,
            },
            Node {
                name: "kind-vitui-worker".to_owned(),
                role: "<none>",
                version: "v1.30.2",
                ready: true,
                capacity: (8000, 16384),
                age: 61 * 1440,
            },
            Node {
                name: "kind-vitui-worker2".to_owned(),
                role: "<none>",
                version: "v1.30.2",
                ready: false,
                capacity: (8000, 16384),
                age: 12 * 1440,
            },
        ];

        let mut deployments = Vec::new();
        let mut pods = Vec::new();
        for (owner, (name, ns, want, image)) in WORKLOADS.iter().enumerate() {
            deployments.push(Deployment {
                ns: *ns,
                name: (*name).to_owned(),
                want: *want,
                age: u32::try_from(hash(name) % (30 * 1440)).unwrap_or(0) + 90,
            });
            for replica in 0..*want {
                let suffix = format!("{name}-{replica}");
                let h = hash(&suffix);
                let phase = match h % 17 {
                    0 => Phase::CrashLoopBackOff,
                    1 => Phase::Pending,
                    2 => Phase::ContainerCreating,
                    3 => Phase::ImagePullBackOff,
                    4 => Phase::Completed,
                    5 => Phase::Error,
                    _ => Phase::Running,
                };
                let pod_name = format!(
                    "{name}-{:x}-{}",
                    (h >> 40) & 0xf_ffff,
                    &"abcdefghijklmnopqrstuvwxyz"[(h % 20) as usize..(h % 20) as usize + 5],
                );
                let containers = vec![
                    Container {
                        name: (*name).to_owned(),
                        image: (*image).to_owned(),
                        ready: phase == Phase::Running,
                        state: phase,
                        restarts: u32::try_from(h % 4).unwrap_or(0),
                        cpu: u32::try_from(h % 480).unwrap_or(0) + 5,
                        mem: u32::try_from(h % 700).unwrap_or(0) + 24,
                    },
                    Container {
                        name: "istio-proxy".to_owned(),
                        image: "docker.io/istio/proxyv2:1.22.3".to_owned(),
                        ready: phase.healthy(),
                        state: phase,
                        restarts: 0,
                        cpu: u32::try_from(h % 40).unwrap_or(0) + 3,
                        mem: u32::try_from(h % 90).unwrap_or(0) + 40,
                    },
                ];
                pods.push(Pod {
                    ns: *ns,
                    owner,
                    phase,
                    restarts: containers.iter().map(|c| c.restarts).sum(),
                    ip: format!("10.244.{}.{}", h % 4, h % 250 + 2),
                    node: usize::try_from(h % 3).unwrap_or(0),
                    age: u32::try_from(h % (14 * 1440)).unwrap_or(0) + 3,
                    qos: if h.is_multiple_of(3) {
                        "Guaranteed"
                    } else {
                        "Burstable"
                    },
                    name: pod_name,
                    containers,
                });
            }
        }

        let services = vec![
            (
                "api-gateway",
                0,
                "LoadBalancer",
                "10.96.14.7",
                "203.0.113.9",
                "80:31380/TCP",
            ),
            (
                "checkout",
                0,
                "ClusterIP",
                "10.96.201.4",
                "<none>",
                "8080/TCP",
            ),
            (
                "kube-dns",
                1,
                "ClusterIP",
                "10.96.0.10",
                "<none>",
                "53/UDP,53/TCP",
            ),
            (
                "prometheus",
                2,
                "NodePort",
                "10.96.77.3",
                "<none>",
                "9090:30090/TCP",
            ),
        ]
        .into_iter()
        .map(|(name, ns, kind, ip, external, ports)| Service {
            ns,
            name: name.to_owned(),
            kind,
            cluster_ip: ip.to_owned(),
            external: external.to_owned(),
            ports: ports.to_owned(),
            age: u32::try_from(hash(name) % (40 * 1440)).unwrap_or(0) + 200,
        })
        .collect();

        Cluster {
            namespaces: NAMESPACES.to_vec(),
            nodes,
            deployments,
            services,
            pods,
        }
    }

    /// Millicores and mebibytes in use across every pod, against what the ready nodes can hold.
    fn pressure(&self) -> ((u32, u32), (u32, u32)) {
        let used = self
            .pods
            .iter()
            .filter(|p| p.phase.healthy())
            .fold((0, 0), |(c, m), p| {
                let (pc, pm) = p.usage();
                (c + pc, m + pm)
            });
        let cap = self
            .nodes
            .iter()
            .filter(|n| n.ready)
            .fold((0, 0), |(c, m), n| (c + n.capacity.0, m + n.capacity.1));
        (used, cap)
    }

    /// How many pods a node is running.
    fn pods_on(&self, node: usize) -> usize {
        self.pods.iter().filter(|p| p.node == node).count()
    }
}

/// `61d`, `13h`, `47m` — k9s's own three units and no fourth.
fn age(minutes: u32, out: &mut String) {
    out.clear();
    let _ = if minutes >= 1440 {
        write!(out, "{}d", minutes / 1440)
    } else if minutes >= 60 {
        write!(out, "{}h", minutes / 60)
    } else {
        write!(out, "{minutes}m")
    };
}

// ── the view stack ───────────────────────────────────────────────────────────────────────────────

/// One level of k9s's view stack. `Esc` pops; the breadcrumbs *are* this stack.
enum View {
    /// Every pod, or the pods of one deployment.
    Pods {
        owner: Option<usize>,
    },
    Deployments,
    Services,
    Nodes,
    Namespaces,
    /// A pod's containers.
    Containers {
        pod: usize,
    },
    /// Logs, YAML or a description: a level like any other, and it scrolls with the same keys.
    Text {
        crumb: String,
        lines: Vec<String>,
        top: usize,
    },
}

impl View {
    /// What the breadcrumb says, and what the panel title is built from.
    fn crumb(&self, cluster: &Cluster) -> String {
        match self {
            View::Pods { owner: None } => "pods".to_owned(),
            View::Pods { owner: Some(d) } => cluster.deployments[*d].name.clone(),
            View::Deployments => "deploys".to_owned(),
            View::Services => "svcs".to_owned(),
            View::Nodes => "nodes".to_owned(),
            View::Namespaces => "ns".to_owned(),
            View::Containers { pod } => cluster.pods[*pod].name.clone(),
            View::Text { crumb, .. } => crumb.clone(),
        }
    }

    /// The word the panel title leads with — k9s capitalises the kind, not the instance.
    fn kind(&self) -> &'static str {
        match self {
            View::Pods { .. } => "Pods",
            View::Deployments => "Deployments",
            View::Services => "Services",
            View::Nodes => "Nodes",
            View::Namespaces => "Namespaces",
            View::Containers { .. } => "Containers",
            View::Text { .. } => "Text",
        }
    }
}

/// k9s's three sorts that have a key each.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sort {
    Name,
    Age,
    Status,
}

// ── the columns, one static set per view ─────────────────────────────────────────────────────────

/// `internal/render/pod.go`'s own order, narrowed to what fits without `Ctrl+W`.
const POD_COLS: [Column; 10] = [
    Column::new(0, "NAMESPACE", Fixed(14)),
    Column::new(1, "NAME", Weight(3)),
    Column::new(2, "READY", Fixed(6)),
    Column::new(3, "STATUS", Fixed(18)),
    Column::new(4, "RESTARTS", Fixed(9)),
    Column::new(5, "CPU", Fixed(6)),
    Column::new(6, "MEM", Fixed(7)),
    Column::new(7, "IP", Fixed(15)),
    Column::new(8, "NODE", Weight(2)),
    Column::new(9, "AGE", Fixed(5)),
];

/// The same, with the two `Wide` columns k9s keeps behind `Ctrl+W`.
const POD_COLS_WIDE: [Column; 12] = [
    Column::new(0, "NAMESPACE", Fixed(14)),
    Column::new(1, "NAME", Weight(3)),
    Column::new(2, "READY", Fixed(6)),
    Column::new(3, "STATUS", Fixed(18)),
    Column::new(4, "RESTARTS", Fixed(9)),
    Column::new(5, "CPU", Fixed(6)),
    Column::new(6, "MEM", Fixed(7)),
    Column::new(7, "IP", Fixed(15)),
    Column::new(8, "NODE", Weight(2)),
    Column::new(10, "QOS", Fixed(11)),
    Column::new(11, "OWNER", Weight(2)),
    Column::new(9, "AGE", Fixed(5)),
];

const DEPLOY_COLS: [Column; 6] = [
    Column::new(0, "NAMESPACE", Fixed(14)),
    Column::new(1, "NAME", Weight(3)),
    Column::new(2, "READY", Fixed(8)),
    Column::new(3, "UP-TO-DATE", Fixed(11)),
    Column::new(4, "AVAILABLE", Fixed(10)),
    Column::new(5, "AGE", Fixed(5)),
];

const SVC_COLS: [Column; 7] = [
    Column::new(0, "NAMESPACE", Fixed(14)),
    Column::new(1, "NAME", Weight(2)),
    Column::new(2, "TYPE", Fixed(13)),
    Column::new(3, "CLUSTER-IP", Fixed(13)),
    Column::new(4, "EXTERNAL-IP", Fixed(13)),
    Column::new(5, "PORTS", Weight(2)),
    Column::new(6, "AGE", Fixed(5)),
];

const NODE_COLS: [Column; 7] = [
    Column::new(0, "NAME", Weight(3)),
    Column::new(1, "STATUS", Fixed(9)),
    Column::new(2, "ROLE", Fixed(15)),
    Column::new(3, "VERSION", Fixed(9)),
    Column::new(4, "PODS", Fixed(6)),
    Column::new(5, "CAPACITY", Fixed(16)),
    Column::new(6, "AGE", Fixed(5)),
];

const NS_COLS: [Column; 4] = [
    Column::new(0, "NAME", Weight(2)),
    Column::new(1, "STATUS", Fixed(9)),
    Column::new(2, "PODS", Fixed(6)),
    Column::new(3, "AGE", Fixed(5)),
];

const CONTAINER_COLS: [Column; 7] = [
    Column::new(0, "NAME", Weight(2)),
    Column::new(1, "IMAGE", Weight(3)),
    Column::new(2, "READY", Fixed(6)),
    Column::new(3, "STATE", Fixed(18)),
    Column::new(4, "RESTARTS", Fixed(9)),
    Column::new(5, "CPU", Fixed(6)),
    Column::new(6, "MEM", Fixed(7)),
];

/// `internal/ui/splash.go`'s `LogoSmall`, verbatim.
const LOGO: [&str; 6] = [
    r" ____  __ ________       ",
    r"|    |/  /   __   \______",
    r"|       /\____    /  ___/",
    r"|    \   \  /    /\___  \",
    r"|____|\__ \/____//____  /",
    r"         \/           \/ ",
];

/// The action mnemonics down the middle of the header, in k9s's own wording.
const ACTIONS: [(&str, &str); 7] = [
    ("<ctrl-d>", "Delete"),
    ("<d>", "Describe"),
    ("<y>", "YAML"),
    ("<l>", "Logs"),
    ("<enter>", "Drill in"),
    ("<ctrl-r>", "Refresh"),
    ("<?>", "Help"),
];

/// What the `?` view lists.
const HELP: [(&str, &str); 16] = [
    (":cmd", "pods · deploy · svc · no · ns · q"),
    ("/filter", "keep the rows whose name or status matches"),
    ("esc", "pop a level, or clear the filter at the root"),
    ("enter", "drill in"),
    ("d", "describe"),
    ("y", "YAML"),
    ("l", "logs"),
    ("space", "mark a row · `ctrl-space` does the same"),
    ("ctrl-\\", "clear the marks"),
    (
        "ctrl-d",
        "delete — tab or ←/→, then enter: it opens on Cancel",
    ),
    ("shift-n / a / s", "sort by name / age / status"),
    ("0 … 5", "the namespace mnemonics"),
    ("ctrl-w", "wide columns"),
    ("ctrl-e / ctrl-g", "the header / the breadcrumbs"),
    ("ctrl-r", "refresh"),
    ("ctrl-q", "quit"),
];

// ── what a prompt is ─────────────────────────────────────────────────────────────────────────────

/// Which prompt is open over the header. k9s has exactly these two.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Prompt {
    /// `:` — a resource name.
    Command,
    /// `/` — a filter over the rows on screen.
    Filter,
}

impl Prompt {
    const fn sigil(self) -> &'static str {
        match self {
            // **k9s draws a dog here and this does not**, for `spf`'s reason one application over:
            // The repertoire ladder is ascii · unicode · extended, the theme names twenty glyphs
            // and none of them is a mascot, and a hard-coded emoji is a wide cluster outside all
            // three rungs. The sigil is the key you pressed.
            Prompt::Command => " : ",
            Prompt::Filter => " / ",
        }
    }

    const fn title(self) -> &'static str {
        match self {
            Prompt::Command => " Command ",
            Prompt::Filter => " Filter ",
        }
    }
}

/// **What a dialog or a prompt decided, written inside a frame and read after it.**
///
/// An overlay body cannot answer its caller and neither can a widget drawn inside the frame, so
/// this is the inbox — `console`'s arrangement, and `commander`'s.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Act {
    /// The command or the filter was submitted.
    Submit,
    /// The prompt was abandoned.
    Cancel,
    /// The delete dialog was confirmed.
    Delete,
    /// The delete dialog, or the help, was dismissed.
    Close,
    /// A text level was scrolled.
    Scroll(i32),
}

// ── the keyboard, as a map ───────────────────────────────────────────────────────────────────────

/// **Every binding this application owns, and it is a [`KeyMap`] rather than a match on `k.code`.**
///
/// The reason is the wire encoding, met the hard way. **One keystroke has four wire spellings** and a
/// terminal speaking the enhanced keyboard protocol picks the last two: `?` arrives as
/// `CSI 47;2;63u` — the base key `/`, the shift bit, and `?` as the text — or as `CSI 47;2u`, which
/// says `/` and shift and nothing about `?` at all. A match on `Code::Char('?')` sees neither, so
/// `?` opened the *filter* and `Shift+N` sorted nothing, on every terminal in this repository's own
/// conform suite. It reads correctly only on a legacy terminal, which is what a headless gate and a
/// hand test in one emulator both are.
///
/// [`Chord::typed`] is the answer for three of the four: it compares the **text the terminal says
/// was produced** and masks the shift bit, because on a character that bit is *how it was made* and
/// not intent. The fourth needs an alternate — `Chord::key('/').shift()` — and it is a second
/// binding rather than a repair, because nothing in the process knows a `?` was produced.
///
/// **Order is precedence** ([`KeyMap::match_first`] takes the first match), and it is load-bearing
/// twice: `?` is bound before `/` and `Shift+N` before anything reading a bare `n`, or the alternate
/// for the shifted key would be swallowed by the unshifted one.
/// **`?`, on every wire spelling a terminal can send it as, in one place.**
///
/// [`key_map`]'s own `shifted` helper is a closure inside that function and is not reachable from a
/// draw, so the help modal's *close* arm re-spelled the pair by hand — a second place for one of the
/// two arms to go missing, in a file written to record that exact defect. Worse, the hand-written
/// version was a `match` on `k.code`, which misses a spelling [`Chord::typed`] catches: a terminal
/// speaking the enhanced protocol may report the base key `/` **and** the text `?`, and
/// `Code::Char('?')` sees neither half of that.
const HELP_KEYS: [Chord; 2] = [Chord::typed('?'), Chord::key('/').shift()];

/// Whether a keystroke is one of [`HELP_KEYS`].
///
/// `MatchMode::Masked` is what ships and what [`KeyMap`] uses, so the two readings of `?` in this
/// file cannot disagree about the lock bits either.
fn is_help_key(k: &Pressed) -> bool {
    HELP_KEYS
        .iter()
        .any(|c| c.matches(k, vitui_runtime::keys::MatchMode::Masked))
}

fn key_map() -> KeyMap {
    let ctrl = |c: char| Chord::key(c).ctrl();
    // A shifted character: what the terminal produced, and the base key beside it for the spelling
    // that produced nothing.
    let shifted = |shift: char, base: char| [Chord::typed(shift), Chord::key(base).shift()];
    let mut map = KeyMap::new()
        .bind(&[ctrl('q')], QUIT, "Quit")
        .bind(&[ctrl('r')], REFRESH, "Refresh")
        .bind(&[ctrl('w')], WIDE, "Wide columns")
        .bind(&[ctrl('e')], HEADER, "Toggle header")
        .bind(&[ctrl('g')], CRUMBS, "Toggle breadcrumbs")
        .bind(&[ctrl('d')], DELETE, "Delete")
        .bind(&[ctrl('\\')], CLEAR, "Clear marks")
        // **`space`, which is k9s's own key, since `crate::collect::owns` landed.** It was
        // `Ctrl+Space` for the life of this port: a bare `Space` was answered with
        // `Gesture::Toggle` in every mode and ignored by `apply` at `Mode::Cursor`, so it was
        // consumed to do nothing and never arrived. The chord stays bound beside it — a port's
        // muscle memory is the point, and nothing else in this application wants it.
        .bind(&[Chord::key(' '), ctrl(' ')], MARK, "Mark")
        // **Before `/`.** `Chord::key('/').shift()` and `Chord::typed('/')` both answer the fourth
        // spelling of `?`, and the first binding wins.
        .bind(&shifted('?', '/'), SHOW_HELP, "Help")
        .bind(&shifted(':', ';'), COMMAND, "Command")
        .bind(&[Chord::typed('/')], FILTER, "Filter")
        .bind(&shifted('N', 'n'), SORT_NAME, "Sort by name")
        .bind(&shifted('A', 'a'), SORT_AGE, "Sort by age")
        .bind(&shifted('S', 's'), SORT_STATUS, "Sort by status")
        .bind(&[Chord::typed('d')], DESCRIBE, "Describe")
        .bind(&[Chord::typed('y')], YAML, "YAML")
        .bind(&[Chord::typed('l')], LOGS, "Logs")
        .bind(&[Chord::new(Code::Enter)], DRILL, "Drill in")
        .bind(&[Chord::new(Code::Escape)], POP, "Back");
    for (i, action) in (NS0..=NS5).enumerate() {
        let digit = char::from(b'0' + u8::try_from(i).unwrap_or(0));
        map = map.bind(&[Chord::typed(digit)], action, "Namespace");
    }
    map
}

const QUIT: ActionId = 1;
const REFRESH: ActionId = 2;
const WIDE: ActionId = 3;
const HEADER: ActionId = 4;
const CRUMBS: ActionId = 5;
const DELETE: ActionId = 6;
const CLEAR: ActionId = 7;
const MARK: ActionId = 8;
const SHOW_HELP: ActionId = 9;
const COMMAND: ActionId = 10;
const FILTER: ActionId = 11;
const SORT_NAME: ActionId = 12;
const SORT_AGE: ActionId = 13;
const SORT_STATUS: ActionId = 14;
const DESCRIBE: ActionId = 15;
const YAML: ActionId = 16;
const LOGS: ActionId = 17;
const DRILL: ActionId = 18;
const POP: ActionId = 19;
/// `<0> all` through `<5> flux-system`, contiguous so the digit is the offset.
const NS0: ActionId = 20;
const NS5: ActionId = 25;

/// **Every [`ActionId`] above is distinct and none of them lands in the digit row, and the compiler
/// says so.**
///
/// Two things a copy-paste breaks silently here, because a wrong verb draws a perfectly good screen
/// and [`KeyMap`] has no reason to object. The first is a plain duplicate. The second is this file's
/// own: `NS0..=NS5` is a **span** — the digit is the offset, so 21, 22, 23 and 24 are ids nothing
/// writes down — and an id landing inside it collides with a namespace nobody would think to look
/// for.
const _: () = {
    let ids = [
        QUIT,
        REFRESH,
        WIDE,
        HEADER,
        CRUMBS,
        DELETE,
        CLEAR,
        MARK,
        SHOW_HELP,
        COMMAND,
        FILTER,
        SORT_NAME,
        SORT_AGE,
        SORT_STATUS,
        DESCRIBE,
        YAML,
        LOGS,
        DRILL,
        POP,
    ];
    let mut i = 0;
    while i < ids.len() {
        assert!(
            ids[i] < NS0 || ids[i] > NS5,
            "an action landed inside the `NS0..=NS5` digit row"
        );
        let mut j = i + 1;
        while j < ids.len() {
            assert!(ids[i] != ids[j], "two actions share one `ActionId`");
            j += 1;
        }
        i += 1;
    }
    assert!(NS0 < NS5, "the digit row runs upward");
};

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Everything this program knows.
struct App {
    cluster: Cluster,
    /// The view stack. Never empty: the root is `Pods`.
    stack: Vec<View>,
    /// `None` is `<0> all`.
    namespace: Option<usize>,
    /// The rows of the top view, filtered and sorted. Rebuilt by [`App::relist`].
    rows: Vec<usize>,
    st: TableState,
    /// The marks, which are this application's store: `Mode::Cursor` never selects, so an arrow
    /// key does not throw away what `space` marked.
    marks: Selection,
    rev: Revision,
    sort: Sort,
    desc: bool,
    wide: bool,
    header: bool,
    crumbs: bool,
    /// The open prompt and its buffer.
    prompt: Option<Prompt>,
    buf: Text,
    /// The standing filter. `/` sets it, `Esc` at the root clears it.
    filter: String,
    /// Whether the delete dialog is up, and over how many rows.
    deleting: Option<usize>,
    /// Whether the help view is up.
    helping: bool,
    /// The flash line k9s puts under the logo.
    flash: String,
    /// The inbox.
    pending: Option<Act>,
    /// **Put the keyboard back on the table.** Set whenever a dialog or a prompt closes: the
    /// focused widget stopped drawing, and the runtime's vanish rule leaves the focus on whatever
    /// survived nearest in the ring — never on the table this application wants it on.
    seat: bool,
    exit: bool,
    /// A scratch buffer for the age column, so a frame formats into one allocation.
    scratch: String,
    /// The bindings, built once. See [`key_map`] for why this is a map and not a match.
    keys: KeyMap,
}

impl App {
    fn new() -> App {
        let mut app = App {
            cluster: Cluster::seed(),
            stack: vec![View::Pods { owner: None }],
            namespace: Some(0),
            rows: Vec::new(),
            st: TableState::new(),
            marks: Selection::new(),
            rev: Revision::UNKNOWN,
            sort: Sort::Name,
            desc: false,
            wide: false,
            header: true,
            crumbs: true,
            prompt: None,
            buf: Text::input(),
            filter: String::new(),
            deleting: None,
            helping: false,
            flash: "k9s on vitui — press ? for the keys".to_owned(),
            pending: None,
            seat: true,
            exit: false,
            scratch: String::with_capacity(16),
            keys: key_map(),
        };
        app.relist();
        app
    }

    /// The view on top of the stack.
    fn top(&self) -> &View {
        self.stack.last().expect("the stack is never empty")
    }

    /// The columns the top view is drawn with.
    fn columns(&self) -> &'static [Column] {
        match self.top() {
            View::Pods { .. } if self.wide => &POD_COLS_WIDE,
            View::Pods { .. } => &POD_COLS,
            View::Deployments => &DEPLOY_COLS,
            View::Services => &SVC_COLS,
            View::Nodes => &NODE_COLS,
            View::Namespaces => &NS_COLS,
            View::Containers { .. } => &CONTAINER_COLS,
            View::Text { .. } => &[],
        }
    }

    /// Rebuild the row list of the top view: the namespace, then the filter, then the sort.
    ///
    /// **One pass and one stamp.** The revision is what lets the table drop a cursor it can no
    /// longer trust rather than keep one pointing at a row that has moved.
    fn relist(&mut self) {
        let keep = self.filter.to_lowercase();
        let ns = self.namespace;
        let cluster = &self.cluster;
        let mut rows: Vec<usize> = match self.stack.last() {
            Some(View::Pods { owner }) => (0..cluster.pods.len())
                .filter(|&i| owner.is_none_or(|d| cluster.pods[i].owner == d))
                .filter(|&i| ns.is_none_or(|n| cluster.pods[i].ns == n))
                .filter(|&i| {
                    keep.is_empty()
                        || cluster.pods[i].name.to_lowercase().contains(&keep)
                        || cluster.pods[i].phase.word().to_lowercase().contains(&keep)
                })
                .collect(),
            Some(View::Deployments) => (0..cluster.deployments.len())
                .filter(|&i| ns.is_none_or(|n| cluster.deployments[i].ns == n))
                .filter(|&i| {
                    keep.is_empty() || cluster.deployments[i].name.to_lowercase().contains(&keep)
                })
                .collect(),
            Some(View::Services) => (0..cluster.services.len())
                .filter(|&i| ns.is_none_or(|n| cluster.services[i].ns == n))
                .filter(|&i| {
                    keep.is_empty() || cluster.services[i].name.to_lowercase().contains(&keep)
                })
                .collect(),
            Some(View::Nodes) => (0..cluster.nodes.len())
                .filter(|&i| {
                    keep.is_empty() || cluster.nodes[i].name.to_lowercase().contains(&keep)
                })
                .collect(),
            Some(View::Namespaces) => (0..cluster.namespaces.len())
                .filter(|&i| keep.is_empty() || cluster.namespaces[i].contains(&keep))
                .collect(),
            Some(View::Containers { pod }) => (0..cluster.pods[*pod].containers.len()).collect(),
            Some(View::Text { .. }) | None => Vec::new(),
        };

        // The sort is over the same three keys k9s gives a shortcut to, and `Status` falls back to
        // the name so that two `Running` pods keep a stable order.
        let key = |i: usize| -> (String, u32, u8) {
            match self.stack.last() {
                Some(View::Pods { .. }) => {
                    let p = &cluster.pods[i];
                    (p.name.clone(), p.age, p.phase as u8)
                }
                Some(View::Deployments) => {
                    let d = &cluster.deployments[i];
                    (d.name.clone(), d.age, 0)
                }
                Some(View::Services) => {
                    let s = &cluster.services[i];
                    (s.name.clone(), s.age, 0)
                }
                Some(View::Nodes) => {
                    let n = &cluster.nodes[i];
                    (n.name.clone(), n.age, u8::from(!n.ready))
                }
                Some(View::Namespaces) => (cluster.namespaces[i].to_owned(), 0, 0),
                Some(View::Containers { pod }) => {
                    let c = &cluster.pods[*pod].containers[i];
                    (c.name.clone(), 0, c.state as u8)
                }
                _ => (String::new(), 0, 0),
            }
        };
        rows.sort_by(|&a, &b| {
            let (ka, kb) = (key(a), key(b));
            let ord = match self.sort {
                Sort::Name => ka.0.cmp(&kb.0),
                Sort::Age => kb.1.cmp(&ka.1).then_with(|| ka.0.cmp(&kb.0)),
                Sort::Status => ka.2.cmp(&kb.2).then_with(|| ka.0.cmp(&kb.0)),
            };
            if self.desc { ord.reverse() } else { ord }
        });

        self.rows = rows;
        self.marks.clear();
        self.st.coll.sel.lead = self.st.coll.sel.lead.min(self.rows.len().saturating_sub(1));
        self.rev = Revision::fresh();
    }
}

// ── the drawing ──────────────────────────────────────────────────────────────────────────────────

impl App {
    /// One frame, top to bottom.
    ///
    /// `&'f mut self` because the delete dialog's overlay body is `+ 'f` and holds this
    /// application's own fields.
    fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        let whole = cx.area();
        let head_rows = match (self.header, self.prompt.is_some()) {
            (_, true) => 3,
            (true, false) => 7,
            (false, false) => 0,
        };
        let crumb_rows = u16::from(self.crumbs);
        let [head, body, crumbs] =
            Col::new().split(whole, [Fixed(head_rows), Weight(1), Fixed(crumb_rows)]);

        match self.prompt {
            Some(which) => self.draw_prompt(cx, head, which),
            None if self.header => self.draw_header(cx, head),
            None => {}
        }
        let table_id = self.draw_body(cx, body);
        if self.crumbs {
            self.draw_crumbs(cx, crumbs);
        }

        // **Nothing holds the focus until this application seats it**, and
        // the prompt takes it away from the table while it is open — which is what stops a `d`
        // meant for the filter being read as *describe*.
        let busy = self.deleting.is_some() || self.helping;
        if self.prompt.is_none()
            && !busy
            && (self.seat || cx.focused().is_none())
            && let Some(id) = table_id
        {
            cx.focus(id);
            self.seat = false;
        }

        self.draw_dialog(cx);
    }

    /// The cluster-info block, the mnemonics and the logo.
    fn draw_header(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let [info, mnemonics, logo] = Row::new().split(area, [Weight(4), Weight(5), Fixed(26)]);
        let ((cpu, mem), (cap_cpu, cap_mem)) = self.cluster.pressure();

        // `internal/view/cluster_info.go`'s seven rows, in its own order.
        let rows = [
            ("Context", "kind-vitui".to_owned()),
            ("Cluster", "kind-vitui".to_owned()),
            ("User", "kubernetes-admin".to_owned()),
            ("K9s Rev", "v0.32.7".to_owned()),
            ("K8s Rev", "v1.30.2".to_owned()),
            ("CPU", format!("{cpu}m / {cap_cpu}m")),
            ("MEM", format!("{mem}Mi / {cap_mem}Mi")),
        ];
        for (i, (label, value)) in rows.iter().enumerate() {
            let y = info.y + i32::try_from(i).unwrap_or(0);
            if y >= info.bottom() {
                break;
            }
            let row = Rect::new(info.x, y, info.w, 1);
            let [key, val] = Row::new().split(row, [Fixed(10), Weight(1)]);
            fit_with(
                cx,
                key,
                &format!(" {label}:"),
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Dim,
                    pad: Role::Dim,
                },
            );
            // The last two rows carry a meter beside the number, which is what k9s's own
            // colour-coded percentage is trying to say and cannot.
            if i >= 5 {
                let (used, cap) = if i == 5 {
                    (cpu, cap_cpu)
                } else {
                    (mem, cap_mem)
                };
                let share = f32::from(u16::try_from(used.min(cap)).unwrap_or(0))
                    / f32::from(u16::try_from(cap.max(1)).unwrap_or(1));
                let [bar, num] = Row::new().split(val, [Fixed(12), Weight(1)]);
                meter_with(
                    cx,
                    bar,
                    share,
                    &MeterOpts {
                        done: if share > 0.8 { Role::Danger } else { Role::Ok },
                        rest: Role::Border,
                        ..Default::default()
                    },
                );
                fit_with(
                    cx,
                    num,
                    &format!(" {:.0}%  {value}", share * 100.0),
                    &FitOpts {
                        justify: Justify::Start,
                        role: Role::Body,
                        pad: Role::Body,
                    },
                );
            } else {
                fit_with(
                    cx,
                    val,
                    value,
                    &FitOpts {
                        justify: Justify::Start,
                        role: Role::Title,
                        pad: Role::Title,
                    },
                );
            }
        }

        // The two mnemonic columns: the namespaces `0`..`4`, and the actions.
        let [ns_col, act_col] = Row::new().split(mnemonics, [Weight(1), Weight(1)]);
        // **Both columns are written to their own height, not to their content's.** A band the
        // application composes is a rectangle the application owes in full: the seventh
        // row of the namespace column has no namespace in it, and left unwritten it keeps whatever
        // the *previous* frame put there — which on this screen was a pod's `Running`.
        let here = self.namespace;
        let names: Vec<&str> = std::iter::once("all").chain(NAMESPACES).collect();
        for i in 0..usize::from(ns_col.h) {
            let row = Rect::new(
                ns_col.x,
                ns_col.y + i32::try_from(i).unwrap_or(0),
                ns_col.w,
                1,
            );
            let chosen = if i == 0 {
                here.is_none()
            } else {
                here == Some(i - 1)
            };
            let role = if chosen { Role::Ok } else { Role::Dim };
            let line = names
                .get(i)
                .map_or_else(String::new, |name| format!(" <{i}> {name}"));
            fit_with(
                cx,
                row,
                &line,
                &FitOpts {
                    justify: Justify::Start,
                    role,
                    pad: role,
                },
            );
        }
        for i in 0..usize::from(act_col.h) {
            let (key, what) = ACTIONS.get(i).copied().unwrap_or(("", ""));
            let row = Rect::new(
                act_col.x,
                act_col.y + i32::try_from(i).unwrap_or(0),
                act_col.w,
                1,
            );
            let [k, w] = Row::new().split(row, [Fixed(10), Weight(1)]);
            fit_with(
                cx,
                k,
                &format!(" {key}"),
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Warn,
                    pad: Role::Warn,
                },
            );
            fit_with(
                cx,
                w,
                what,
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Dim,
                    pad: Role::Dim,
                },
            );
        }

        // The logo, and the flash line under it — which is where k9s puts its status.
        for (i, art) in LOGO.iter().enumerate().take(usize::from(logo.h)) {
            let row = Rect::new(logo.x, logo.y + i32::try_from(i).unwrap_or(0), logo.w, 1);
            fit_with(
                cx,
                row,
                art,
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Warn,
                    pad: Role::Warn,
                },
            );
        }
        if logo.h > 6 {
            let row = Rect::new(logo.x, logo.y + 6, logo.w, 1);
            fit_with(
                cx,
                row,
                &self.flash,
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Ok,
                    pad: Role::Ok,
                },
            );
        }
    }

    /// The `:` or `/` prompt, which replaces the header while it is open.
    fn draw_prompt(&mut self, cx: &mut Ctx<'_, '_>, area: Rect, which: Prompt) {
        let panel = panel_with(
            cx,
            area,
            which.title(),
            &PanelOpts {
                border: Role::Focus,
                padded: false,
                ..Default::default()
            },
        );
        let sigil = which.sigil();
        let cut = u16::try_from(sigil.chars().count() + 1)
            .unwrap_or(4)
            .min(panel.interior.w);
        let (mark, entry) = rect::split_at_h(panel.interior, cut);
        fit_with(
            cx,
            mark,
            sigil,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Warn,
                pad: Role::Warn,
            },
        );
        let resp = field(cx, entry, &mut self.buf);
        if !cx.is_focused(resp.id) {
            cx.focus(resp.id);
        }
        // **`Enter` and `Esc` are read after the frame and not here**, and that is a fact about
        // the router rather than a style: `Ctx::next_key` answers *only the focused id*, so a sink
        // beside a focused field is deaf. What the field declines goes back on the queue and
        // reaches `Driver::unhandled` — see `App::take_unhandled`.
    }

    /// The resource table, or a text level. Answers the id the keyboard should sit on.
    fn draw_body(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> Option<vitui_runtime::Id> {
        let title = self.title();
        let panel = panel_with(
            cx,
            area,
            &title,
            &PanelOpts {
                border: Role::Border,
                padded: false,
                ..Default::default()
            },
        );
        if panel.interior.is_empty() {
            return None;
        }
        if matches!(self.top(), View::Text { .. }) {
            return Some(self.draw_text_level(cx, panel.interior));
        }

        let cols = self.columns();
        let opts = TableOpts {
            header: true,
            coll: CollOpts {
                // **A cursor that never selects**, so that an arrow key does not throw away what
                // `space` marked. k9s's marks behave exactly this way.
                mode: Mode::Cursor,
                ..CollOpts::default()
            },
        };
        let App {
            cluster,
            rows,
            marks,
            st,
            rev,
            scratch,
            stack,
            ..
        } = self;
        let view = stack.last().expect("the stack is never empty");
        let resp = table(
            cx,
            panel.interior,
            st,
            &opts,
            cols,
            Rows::new(rows.len(), *rev),
            // **The type-ahead must never match**: `d`, `l` and `y` are k9s's verbs, and a search
            // that answered would eat them. See this file's header.
            &mut |_buf, _range| None,
            &mut |cx, r, cell: Cell, face: Face| {
                let Some(&at) = rows.get(cell.row) else {
                    return;
                };
                let face = Face {
                    selected: marks.contains(cell.row),
                    ..face
                };
                let base = face_role(face);
                let (text, justify, own) = cell_of(cluster, view, at, cell.key, scratch);
                let role = if base == Role::Body { own } else { base };
                cell_into(cx, r, &text, justify, role);
            },
        );
        // **A double click would be `Enter`, and it was not observed to arrive.** The component
        // declares `Interest::CLICK` and returns `cx.scrollable`'s own `Response`, so the field is
        // wired; two clicks 30, 80 and 250 ms apart on one row — all inside the 400 ms threshold —
        // report `clicked` and never `double_clicked`. It is left connected rather than deleted
        // because the day it fires this is the line that wanted it, and the keyboard route is the
        // documented one.
        let drill = resp.double_clicked;
        if drill {
            self.drill();
        }
        Some(resp.id)
    }

    /// A logs, YAML or describe level: the lines, a scrollbar and the same keys the table has.
    fn draw_text_level(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> vitui_runtime::Id {
        let [text, bar] = Row::new().split(area, [Weight(1), Fixed(1)]);
        let View::Text { lines, top, .. } = self.top() else {
            unreachable!("the caller checked")
        };
        let (lines, top) = (lines, *top);
        for i in 0..usize::from(text.h) {
            let row = Rect::new(text.x, text.y + i32::try_from(i).unwrap_or(0), text.w, 1);
            let line = lines.get(top + i).map_or("", String::as_str);
            // A log line that says `error` or `warn` is the one a reader is looking for, and k9s
            // colours it. A `Role` is how that is said here.
            let role = if line.contains("level=error") {
                Role::Danger
            } else if line.contains("level=warn") {
                Role::Warn
            } else {
                Role::Body
            };
            fit_with(
                cx,
                row,
                line,
                &FitOpts {
                    justify: Justify::Start,
                    role,
                    pad: role,
                },
            );
        }
        scrollbar(
            cx,
            bar,
            Span {
                viewport: u32::from(text.h),
                extent: u32::try_from(lines.len()).unwrap_or(0),
                offset: u32::try_from(top).unwrap_or(0),
            },
        );
        let id = cx.id();
        let resp = cx.interact(id, area, Interest::FOCUS.with(Interest::SCROLL));
        // **The notch is added, never negated, and never multiplied.** `crate::collect` establishes
        // the sign — `offset + resp.scrolled.1` — and `crate::input`'s field carries the warning in as
        // many words: *a field that negated it would scroll the wrong way on a screen where every
        // other scrollable widget scrolls the right one.* One notch is one row; how many notches a
        // physical detent produces is the terminal's answer and the engine has already applied it.
        let mut step = resp.scrolled.1;
        while let Some(k) = cx.next_key(id) {
            match k.code {
                Code::Down => step += 1,
                Code::Up => step -= 1,
                Code::PageDown => step += i32::from(text.h),
                Code::PageUp => step -= i32::from(text.h),
                Code::Home => step = i32::MIN / 2,
                Code::End => step = i32::MAX / 2,
                _ => cx.decline(k),
            }
        }
        if step != 0 {
            self.pending = Some(Act::Scroll(step));
        }
        id
    }

    /// The breadcrumb trail, one [`chip_with`] a level, and the marks count at the right.
    fn draw_crumbs(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let mut x = area.x;
        let last = self.stack.len() - 1;
        for (i, view) in self.stack.iter().enumerate() {
            let label = format!(" <{}> ", view.crumb(&self.cluster));
            let w = u16::try_from(label.chars().count()).unwrap_or(0);
            if x + i32::from(w) > area.right() {
                break;
            }
            let cell = Rect::new(x, area.y, w, 1);
            // **A chip and not a label**: the trail is what k9s makes clickable, and a chip is the
            // component that declares a region for one.
            // **One `chip_with` call site is one id, however many crumbs there are.** `with_key`
            // is what makes the trail a row of widgets rather than one widget wearing the last
            // crumb's rectangle — the trap the two file panels met, met again by a loop, and the
            // pointer is where it hides: the hover looks right on whichever chip is drawn last.
            let resp = cx.with_key(i as u64, |cx| {
                chip_with(
                    cx,
                    cell,
                    &label,
                    &ChipOpts {
                        faces: vitui_components::state::Faces {
                            rest: if i == last {
                                Role::Selection
                            } else {
                                Role::Face
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
            });
            if resp.clicked && i < last {
                self.pending = Some(Act::Close);
            }
            x += i32::from(w) + 1;
        }
        let tail = Rect::new(x, area.y, u16::try_from(area.right() - x).unwrap_or(0), 1);
        let marks = self.marks.count();
        let right = match (marks, self.filter.is_empty()) {
            (0, true) => String::new(),
            (0, false) => format!("filter: {}  ", self.filter),
            (n, true) => format!("{n} marked  "),
            (n, false) => format!("{n} marked · filter: {}  ", self.filter),
        };
        fit_with(
            cx,
            tail,
            &right,
            &FitOpts {
                justify: Justify::End,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );
    }

    /// `Pods(default)[9]`, which is k9s's own title format.
    fn title(&self) -> String {
        let scope = match (self.top(), self.namespace) {
            (View::Nodes | View::Namespaces, _) => "all".to_owned(),
            (View::Containers { pod }, _) => self.cluster.pods[*pod].name.clone(),
            (View::Text { crumb, .. }, _) => crumb.clone(),
            (_, None) => "all".to_owned(),
            (_, Some(n)) => self.cluster.namespaces[n].to_owned(),
        };
        match self.top() {
            View::Text { lines, .. } => format!(" {scope}[{}] ", lines.len()),
            view => format!(" {}({scope})[{}] ", view.kind(), self.rows.len()),
        }
    }

    /// The delete confirmation and the help, which are the only two modals here.
    fn draw_dialog<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        let screen = cx.area();
        let theme = *cx.theme();
        let host = cx.id();
        let App {
            deleting,
            helping,
            pending,
            ..
        } = self;
        let (size, title) = match (*deleting, *helping) {
            (Some(_), _) => ((54.min(screen.w), 7.min(screen.h)), " Delete "),
            (None, true) => ((66.min(screen.w), 20.min(screen.h)), " Help "),
            (None, false) => return,
        };
        let opts = OverlayOpts {
            z: Z::MODAL,
            ..OverlayOpts::modal(size.0, size.1, &theme)
        };
        let anchor = Stack::center(screen, size.0, size.1);
        cx.overlay(host, anchor, opts, move |cx| {
            let area = cx.area();
            let shell = ShellOpts {
                kind: ShellKind::Dialog,
                ..ShellOpts::default()
            };
            let _ = overlay_with(cx, area, u32::from(area.h), &shell, &mut |cx, r| {
                let frame = panel_with(
                    cx,
                    r,
                    title,
                    &PanelOpts {
                        border: Role::Focus,
                        title_role: if deleting.is_some() {
                            Role::Danger
                        } else {
                            Role::Title
                        },
                        padded: false,
                        ..Default::default()
                    },
                );
                match *deleting {
                    Some(n) => draw_delete(cx, frame.interior, n, pending),
                    None => draw_help(cx, frame.interior, pending),
                }
            });
        });
    }
}

/// The delete confirmation.
fn draw_delete(cx: &mut Ctx<'_, '_>, r: Rect, n: usize, pending: &mut Option<Act>) {
    let [what, _gap, buttons] = Col::new().split(r, [Fixed(1), Fixed(1), Weight(1)]);
    fit_with(
        cx,
        what,
        &format!(
            "  Delete {n} resource{}? Nothing real is touched.",
            if n == 1 { "" } else { "s" }
        ),
        &FitOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
        },
    );
    let [yes, no] = Row::new()
        .spacing(2)
        .margin(1)
        .split(buttons, [Weight(1), Weight(1)]);
    let yes = button_with(cx, yes, "Delete", &ButtonOpts::default());
    let no = button_with(cx, no, "Cancel", &ButtonOpts::default());
    if yes.clicked {
        *pending = Some(Act::Delete);
    }
    if no.clicked {
        *pending = Some(Act::Close);
    }
    // **`Cancel` is where the keyboard starts, and saying so takes naming both.** A `Kind::Dialog`
    // opens a keyboard trap and the trap seats a stop of its own on the first frame, so
    // `focused().is_none()` is already false — and `Enter` would land on whichever button the walk
    // reached first, which for a delete is the wrong default. k9s makes the same decision by
    // requiring `Tab` before `Enter`.
    if ![yes.id, no.id].iter().any(|id| cx.is_focused(*id)) {
        cx.focus(no.id);
    }
    // **`←` and `→` walk the two buttons, and they must ask for the frame that draws the move.**
    // `Tab` is the ring's own walk and the runtime resolves it in `settle`; an explicit
    // `Ctx::focus` sets the *next* frame's focus, and the buttons above have already drawn with
    // this frame's. Without the request the highlight arrives on whatever key comes next, which
    // reads as an arrow that does nothing — the shape `Frame::resolve_tab` is named for. A
    // horizontal row of buttons is the one place a mouse-only answer is plainly wrong.
    for (id, other, act) in [(yes.id, no.id, Act::Delete), (no.id, yes.id, Act::Close)] {
        while let Some(k) = cx.next_key(id) {
            match k.code {
                Code::Enter | Code::Char(' ') => *pending = Some(act),
                Code::Escape => *pending = Some(Act::Close),
                Code::Left | Code::Right => {
                    cx.focus(other);
                    cx.request_frame();
                }
                _ => cx.decline(k),
            }
        }
    }
}

/// The `?` view.
fn draw_help(cx: &mut Ctx<'_, '_>, r: Rect, pending: &mut Option<Act>) {
    for (i, (key, what)) in HELP.iter().enumerate().take(usize::from(r.h)) {
        let row = Rect::new(r.x, r.y + i32::try_from(i).unwrap_or(0), r.w, 1);
        let [k, w] = Row::new().split(row, [Fixed(18), Weight(1)]);
        fit_with(
            cx,
            k,
            &format!(" {key}"),
            &FitOpts {
                justify: Justify::Start,
                role: Role::Warn,
                pad: Role::Warn,
            },
        );
        fit_with(
            cx,
            w,
            what,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Body,
                pad: Role::Body,
            },
        );
    }
    let id = cx.id();
    let _ = cx.interact(id, r, Interest::FOCUS);
    if cx.focused().is_none() {
        cx.focus(id);
    }
    while let Some(k) = cx.next_key(id) {
        // **The alternate is here too, and read from one home.** `?` on a terminal that reports
        // the base key arrives as `/` with the shift bit, so the modal that `?` opened could not be
        // closed by pressing it again — see [`HELP_KEYS`], which is why this is a call and not a
        // second pair of match arms.
        if matches!(k.code, Code::Escape | Code::Enter) || is_help_key(&k) {
            *pending = Some(Act::Close);
        } else {
            cx.decline(k);
        }
    }
}

/// Write one cell, keeping a one-cell gutter on the right of a left-justified one.
///
/// **A cell drawer owes its whole rectangle**, so the gutter is *written* rather than
/// left out: a left-justified name that fills its column otherwise runs into the column beside it,
/// and the ellipsis lands against a digit. A right-justified cell carries its gutter in the text,
/// because its padding is on the left.
fn cell_into(cx: &mut Ctx<'_, '_>, r: Rect, text: &str, justify: Justify, role: Role) {
    let opts = FitOpts {
        justify,
        role,
        pad: role,
    };
    match justify {
        Justify::Start if r.w > 1 => {
            let (body, gap) = rect::split_at_h(r, r.w - 1);
            fit_with(cx, body, text, &opts);
            fit_with(cx, gap, "", &opts);
        }
        _ => {
            fit_with(cx, r, text, &opts);
        }
    }
}

/// One cell: its text, how it sits in its rectangle, and the role it wants when nothing else
/// claims the row.
///
/// **One function and not one per view**, because the alternative is six row drawers that drift:
/// the column key is the same vocabulary in all of them, and a key a view has no answer for is an
/// empty cell rather than a panic.
fn cell_of(
    cluster: &Cluster,
    view: &View,
    at: usize,
    key: u16,
    scratch: &mut String,
) -> (String, Justify, Role) {
    let start = Justify::Start;
    let end = Justify::End;
    match view {
        View::Pods { .. } => {
            let p = &cluster.pods[at];
            let (ready, of) = p.ready();
            let (cpu, mem) = p.usage();
            match key {
                0 => (cluster.namespaces[p.ns].to_owned(), start, Role::Dim),
                1 => (p.name.clone(), start, Role::Body),
                2 => (
                    format!("{ready}/{of}"),
                    start,
                    if ready == of { Role::Body } else { Role::Warn },
                ),
                3 => (p.phase.word().to_owned(), start, p.phase.role()),
                4 => (
                    format!("{} ", p.restarts),
                    end,
                    if p.restarts > 0 {
                        Role::Warn
                    } else {
                        Role::Body
                    },
                ),
                5 => (format!("{cpu} "), end, Role::Body),
                6 => (format!("{mem} "), end, Role::Body),
                7 => (p.ip.clone(), start, Role::Dim),
                8 => (cluster.nodes[p.node].name.clone(), start, Role::Dim),
                10 => (p.qos.to_owned(), start, Role::Dim),
                11 => (cluster.deployments[p.owner].name.clone(), start, Role::Dim),
                _ => {
                    age(p.age, scratch);
                    (format!("{scratch} "), end, Role::Dim)
                }
            }
        }
        View::Deployments => {
            let d = &cluster.deployments[at];
            let up = cluster
                .pods
                .iter()
                .filter(|p| p.owner == at && p.phase.healthy())
                .count();
            match key {
                0 => (cluster.namespaces[d.ns].to_owned(), start, Role::Dim),
                1 => (d.name.clone(), start, Role::Body),
                2 => (
                    format!("{up}/{}", d.want),
                    start,
                    if u32::try_from(up).unwrap_or(0) == d.want {
                        Role::Ok
                    } else {
                        Role::Warn
                    },
                ),
                3 => (format!("{} ", d.want), end, Role::Body),
                4 => (format!("{up} "), end, Role::Body),
                _ => {
                    age(d.age, scratch);
                    (format!("{scratch} "), end, Role::Dim)
                }
            }
        }
        View::Services => {
            let s = &cluster.services[at];
            match key {
                0 => (cluster.namespaces[s.ns].to_owned(), start, Role::Dim),
                1 => (s.name.clone(), start, Role::Body),
                2 => (s.kind.to_owned(), start, Role::Body),
                3 => (s.cluster_ip.clone(), start, Role::Dim),
                4 => (
                    s.external.clone(),
                    start,
                    if s.external == "<none>" {
                        Role::Dim
                    } else {
                        Role::Ok
                    },
                ),
                5 => (s.ports.clone(), start, Role::Dim),
                _ => {
                    age(s.age, scratch);
                    (format!("{scratch} "), end, Role::Dim)
                }
            }
        }
        View::Nodes => {
            let n = &cluster.nodes[at];
            match key {
                0 => (n.name.clone(), start, Role::Body),
                1 => (
                    if n.ready { "Ready" } else { "NotReady" }.to_owned(),
                    start,
                    if n.ready { Role::Ok } else { Role::Danger },
                ),
                2 => (n.role.to_owned(), start, Role::Dim),
                3 => (n.version.to_owned(), start, Role::Dim),
                4 => (format!("{} ", cluster.pods_on(at)), end, Role::Body),
                5 => (
                    format!("{}m {}Mi", n.capacity.0, n.capacity.1),
                    start,
                    Role::Dim,
                ),
                _ => {
                    age(n.age, scratch);
                    (format!("{scratch} "), end, Role::Dim)
                }
            }
        }
        View::Namespaces => {
            let name = cluster.namespaces[at];
            match key {
                0 => (name.to_owned(), start, Role::Body),
                1 => ("Active".to_owned(), start, Role::Ok),
                2 => (
                    format!("{} ", cluster.pods.iter().filter(|p| p.ns == at).count()),
                    end,
                    Role::Body,
                ),
                _ => ("61d ".to_owned(), end, Role::Dim),
            }
        }
        View::Containers { pod } => {
            let c = &cluster.pods[*pod].containers[at];
            match key {
                0 => (c.name.clone(), start, Role::Body),
                1 => (c.image.clone(), start, Role::Dim),
                2 => (
                    if c.ready { "true" } else { "false" }.to_owned(),
                    start,
                    if c.ready { Role::Ok } else { Role::Warn },
                ),
                3 => (c.state.word().to_owned(), start, c.state.role()),
                4 => (
                    format!("{} ", c.restarts),
                    end,
                    if c.restarts > 0 {
                        Role::Warn
                    } else {
                        Role::Body
                    },
                ),
                5 => (format!("{} ", c.cpu), end, Role::Body),
                _ => (format!("{} ", c.mem), end, Role::Body),
            }
        }
        View::Text { .. } => (String::new(), start, Role::Body),
    }
}

// ── what happens between two frames ──────────────────────────────────────────────────────────────

impl App {
    /// **The inbox, read after the draw.**
    fn answer(&mut self) -> bool {
        let Some(act) = self.pending.take() else {
            return false;
        };
        // Every one of these ends with something that had the keyboard no longer drawing.
        self.seat = true;
        match act {
            Act::Submit => self.submit(),
            Act::Cancel => {
                self.prompt = None;
                self.buf = Text::input();
            }
            Act::Delete => self.delete(),
            Act::Close => {
                if self.helping {
                    self.helping = false;
                } else if self.deleting.is_some() {
                    self.deleting = None;
                } else {
                    self.pop();
                }
            }
            Act::Scroll(by) => {
                if let Some(View::Text { lines, top, .. }) = self.stack.last_mut() {
                    let last = i64::try_from(lines.len().saturating_sub(1)).unwrap_or(0);
                    let to = i64::from(by) + i64::try_from(*top).unwrap_or(0);
                    *top = usize::try_from(to.clamp(0, last)).unwrap_or(0);
                }
            }
        }
        true
    }

    /// What the prompt asked for.
    fn submit(&mut self) {
        let text = self.buf.text().trim().to_owned();
        let which = self.prompt.take();
        self.buf = Text::input();
        match which {
            Some(Prompt::Filter) => {
                self.filter = text;
                self.relist();
                self.flash = if self.filter.is_empty() {
                    "filter cleared".to_owned()
                } else {
                    format!("filtering on {}", self.filter)
                };
            }
            Some(Prompt::Command) => self.command(&text),
            None => {}
        }
    }

    /// `:pods`, `:deploy`, `:svc`, `:no`, `:ns`, `:q` — the aliases k9s answers to.
    fn command(&mut self, text: &str) {
        let (verb, rest) = text.split_once(' ').unwrap_or((text, ""));
        let view = match verb {
            "q" | "quit" => {
                self.exit = true;
                return;
            }
            "po" | "pod" | "pods" => View::Pods { owner: None },
            "dp" | "deploy" | "deployment" | "deployments" => View::Deployments,
            "svc" | "service" | "services" => View::Services,
            "no" | "node" | "nodes" => View::Nodes,
            "ns" | "namespace" | "namespaces" => View::Namespaces,
            "" => return,
            other => {
                self.flash = format!("no such resource: {other}");
                return;
            }
        };
        // `:pod kube-system` is k9s's *view this resource in this namespace*, and it is one split.
        if !rest.is_empty() {
            self.namespace = self.cluster.namespaces.iter().position(|n| *n == rest);
            if self.namespace.is_none() && rest != "all" {
                self.flash = format!("no such namespace: {rest}");
                return;
            }
        }
        self.stack.truncate(1);
        self.stack[0] = view;
        self.filter.clear();
        self.st.coll.offset = 0;
        self.st.coll.sel.lead = 0;
        self.relist();
        self.flash = format!("{} — {} rows", self.title().trim(), self.rows.len());
    }

    /// `Enter`: push the level under the cursor.
    fn drill(&mut self) {
        let Some(&at) = self.rows.get(self.st.coll.sel.lead) else {
            return;
        };
        let next = match self.top() {
            View::Pods { .. } => View::Containers { pod: at },
            View::Deployments => View::Pods { owner: Some(at) },
            View::Namespaces => {
                self.namespace = Some(at);
                self.stack.truncate(1);
                self.stack[0] = View::Pods { owner: None };
                self.st.coll.sel.lead = 0;
                self.st.coll.offset = 0;
                self.relist();
                self.flash = format!("namespace {}", self.cluster.namespaces[at]);
                return;
            }
            View::Nodes => {
                self.flash = format!(
                    "{} runs {} pods",
                    self.cluster.nodes[at].name,
                    self.cluster.pods_on(at)
                );
                return;
            }
            View::Services | View::Containers { .. } | View::Text { .. } => return,
        };
        self.push(next);
    }

    /// Push a level and reset what a level owns.
    fn push(&mut self, view: View) {
        self.stack.push(view);
        self.filter.clear();
        self.st.coll.offset = 0;
        self.st.coll.sel.lead = 0;
        self.relist();
    }

    /// `Esc`: pop a level, or clear the filter at the root.
    fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.st.coll.offset = 0;
            self.st.coll.sel.lead = 0;
            self.relist();
        } else if !self.filter.is_empty() {
            self.filter.clear();
            self.relist();
            self.flash = "filter cleared".to_owned();
        }
    }

    /// The rows `Ctrl+D` acts on: the marks, or the cursor when nothing is marked.
    fn targets(&self) -> Vec<usize> {
        let marked: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(i, _)| self.marks.contains(*i))
            .map(|(_, at)| *at)
            .collect();
        if marked.is_empty() {
            self.rows
                .get(self.st.coll.sel.lead)
                .copied()
                .into_iter()
                .collect()
        } else {
            marked
        }
    }

    /// Carry out the delete. **Only pods can go**, which is honest: a deployment that lost its
    /// pods would grow them back, and modelling that is a controller rather than a screen.
    fn delete(&mut self) {
        let targets = self.targets();
        self.deleting = None;
        if !matches!(self.top(), View::Pods { .. }) {
            self.flash = "this port only deletes pods".to_owned();
            return;
        }
        let mut gone = 0;
        for at in targets {
            if at < self.cluster.pods.len() {
                self.cluster.pods[at].phase = Phase::Terminating;
                gone += 1;
            }
        }
        self.relist();
        self.flash = format!("{gone} pod(s) terminating");
    }

    /// `Ctrl+R`: new metrics, one minute older, and one pod's luck changed.
    fn refresh(&mut self) {
        let salt = u64::from(self.cluster.pods.len() as u32) ^ hash(&self.flash);
        for (i, pod) in self.cluster.pods.iter_mut().enumerate() {
            pod.age += 1;
            for c in &mut pod.containers {
                let h = hash(&c.name).wrapping_mul(salt | 1).wrapping_add(i as u64);
                c.cpu = u32::try_from(h % 480).unwrap_or(0) + 5;
                c.mem = u32::try_from(h % 700).unwrap_or(0) + 24;
            }
        }
        self.relist();
        self.flash = "refreshed".to_owned();
    }

    /// The lines a text level shows. Deterministic in the pod's own name.
    fn text_level(&self, kind: char) -> Option<View> {
        let &at = self.rows.get(self.st.coll.sel.lead)?;
        let View::Pods { .. } = self.top() else {
            return None;
        };
        let pod = &self.cluster.pods[at];
        let (crumb, lines) = match kind {
            'l' => (format!("logs:{}", pod.name), logs_of(pod)),
            'y' => (format!("yaml:{}", pod.name), yaml_of(&self.cluster, pod)),
            _ => (
                format!("describe:{}", pod.name),
                describe_of(&self.cluster, pod),
            ),
        };
        Some(View::Text {
            crumb,
            lines,
            top: 0,
        })
    }

    /// **What nothing wanted, read from the frame that has just drawn.**
    ///
    /// Every letter k9s binds is here rather than in a `KeyMap`, because the table declines them
    /// and this is the one window onto what it declined.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        // **The quit chord is answered before anything else claims the keyboard.** Every branch
        // below returns early for whatever is open, and an application whose way out depends on
        // which dialog is up is an application people cannot leave.
        for k in keys {
            if self.keys.match_first(k) == Some(QUIT) {
                self.exit = true;
                return;
            }
        }
        // **The prompt's two ways out arrive here**, because the field holds the focus and a sink
        // that is not focused is never routed to. Everything else the field already took.
        if self.prompt.is_some() {
            for k in keys {
                match k.code {
                    Code::Enter => self.pending = Some(Act::Submit),
                    Code::Escape => self.pending = Some(Act::Cancel),
                    _ => {}
                }
            }
            return;
        }
        if self.deleting.is_some() || self.helping {
            return;
        }
        for k in keys {
            let Some(action) = self.keys.match_first(k) else {
                continue;
            };
            self.act(action);
        }
    }

    /// One action, whichever chord produced it.
    fn act(&mut self, action: ActionId) {
        match action {
            QUIT => self.exit = true,
            REFRESH => self.refresh(),
            WIDE => {
                self.wide = !self.wide;
                self.st.hoff = 0;
                self.flash = format!("wide columns {}", if self.wide { "on" } else { "off" });
            }
            HEADER => self.header = !self.header,
            CRUMBS => self.crumbs = !self.crumbs,
            DELETE => {
                let n = self.targets().len();
                if n == 0 {
                    self.flash = "nothing to delete".to_owned();
                } else {
                    self.deleting = Some(n);
                }
            }
            CLEAR => {
                self.marks.clear();
                self.flash = "marks cleared".to_owned();
            }
            // **`space`, and `Ctrl+Space` beside it, which is a component finding rather than a
            // taste.** k9s marks with `space`; `crate::collect::from_key` answers a bare `Space`
            // with `Gesture::Toggle` in **every** mode, and `apply` then ignores it at
            // `Mode::Cursor` — so the key was consumed to do nothing and never reached this
            // window, and the chord was bound because a chord is declined
            // (`crate::keys::is_chord`) and arrives. `crate::collect::owns` narrowed the gesture to
            // the modes `apply` acts in, so the bare key arrives too and the port has its own.
            MARK => {
                let lead = self.st.coll.sel.lead;
                let len = self.rows.len();
                if lead < len {
                    // The published `apply`, at the mode a multi-select uses: the store is this
                    // application's and the thirteen arms are the component's.
                    apply(Mode::Multi, &mut self.marks, len, Gesture::Toggle(lead));
                    self.flash = format!("{} marked", self.marks.count());
                }
            }
            SHOW_HELP => self.helping = true,
            COMMAND => self.prompt = Some(Prompt::Command),
            FILTER => {
                self.buf = Text::input();
                self.buf.edit(60, &self.filter);
                self.prompt = Some(Prompt::Filter);
            }
            SORT_NAME => self.sort_by(Sort::Name),
            SORT_AGE => self.sort_by(Sort::Age),
            SORT_STATUS => self.sort_by(Sort::Status),
            DESCRIBE | YAML | LOGS => {
                let kind = match action {
                    YAML => 'y',
                    LOGS => 'l',
                    _ => 'd',
                };
                match self.text_level(kind) {
                    Some(view) => self.push(view),
                    None => self.flash = "nothing to open here".to_owned(),
                }
            }
            DRILL => self.drill(),
            POP => self.pop(),
            NS0..=NS5 => {
                let n = usize::try_from(action - NS0).unwrap_or(0);
                self.namespace = if n == 0 { None } else { Some(n - 1) };
                self.relist();
                self.flash = match self.namespace {
                    None => "namespace: all".to_owned(),
                    Some(n) => format!("namespace: {}", self.cluster.namespaces[n]),
                };
            }
            _ => {}
        }
    }

    /// One of the three sorts, or the same one reversed — which is what pressing it twice does.
    fn sort_by(&mut self, sort: Sort) {
        if self.sort == sort {
            self.desc = !self.desc;
        } else {
            self.sort = sort;
            self.desc = false;
        }
        self.relist();
        self.flash = format!(
            "sorted by {} {}",
            match sort {
                Sort::Name => "name",
                Sort::Age => "age",
                Sort::Status => "status",
            },
            if self.desc { "desc" } else { "asc" },
        );
    }
}

/// A pod's invented log, in the shape a Go service writes.
fn logs_of(pod: &Pod) -> Vec<String> {
    const EVENTS: [(&str, &str); 6] = [
        ("info", "listening on :8080"),
        ("info", "reconcile complete in 41ms"),
        ("warn", "upstream slow: p99=812ms"),
        ("info", "cache hit ratio 0.94"),
        ("error", "dial tcp 10.96.201.4:8080: i/o timeout"),
        ("info", "health probe ok"),
    ];
    let n = 40 + usize::try_from(hash(&pod.name) % 120).unwrap_or(0);
    (0..n)
        .map(|i| {
            let h = hash(&format!("{}:{i}", pod.name));
            let (level, what) = EVENTS[usize::try_from(h % 6).unwrap_or(0)];
            // **The clock runs forward.** A log whose timestamps are a hash of the line number is
            // a log nobody believes, and the whole argument for invented data is that it has to
            // read like the real thing.
            let second = u32::try_from(i).unwrap_or(0) * 7 + u32::try_from(h % 5).unwrap_or(0);
            format!(
                "2026-09-05T{:02}:{:02}:{:02}Z level={level} pod={} msg=\"{what}\"",
                9 + second / 3600 % 12,
                second / 60 % 60,
                second % 60,
                pod.name,
            )
        })
        .collect()
}

/// A pod's invented manifest.
fn yaml_of(cluster: &Cluster, pod: &Pod) -> Vec<String> {
    let mut out = vec![
        "apiVersion: v1".to_owned(),
        "kind: Pod".to_owned(),
        "metadata:".to_owned(),
        format!("  name: {}", pod.name),
        format!("  namespace: {}", cluster.namespaces[pod.ns]),
        "  labels:".to_owned(),
        format!("    app: {}", cluster.deployments[pod.owner].name),
        "spec:".to_owned(),
        format!("  nodeName: {}", cluster.nodes[pod.node].name),
        "  containers:".to_owned(),
    ];
    for c in &pod.containers {
        out.push(format!("  - name: {}", c.name));
        out.push(format!("    image: {}", c.image));
        out.push("    resources:".to_owned());
        out.push("      requests:".to_owned());
        out.push(format!("        cpu: {}m", c.cpu));
        out.push(format!("        memory: {}Mi", c.mem));
    }
    out.push("status:".to_owned());
    out.push(format!("  phase: {}", pod.phase.word()));
    out.push(format!("  podIP: {}", pod.ip));
    out.push(format!("  qosClass: {}", pod.qos));
    out
}

/// A pod's invented description.
fn describe_of(cluster: &Cluster, pod: &Pod) -> Vec<String> {
    let (ready, of) = pod.ready();
    let mut out = vec![
        format!("Name:         {}", pod.name),
        format!("Namespace:    {}", cluster.namespaces[pod.ns]),
        format!("Node:         {}", cluster.nodes[pod.node].name),
        format!("Status:       {}", pod.phase.word()),
        format!("IP:           {}", pod.ip),
        format!(
            "Controlled By: Deployment/{}",
            cluster.deployments[pod.owner].name
        ),
        format!("QoS Class:    {}", pod.qos),
        format!("Ready:        {ready}/{of}"),
        String::new(),
        "Containers:".to_owned(),
    ];
    for c in &pod.containers {
        out.push(format!("  {}:", c.name));
        out.push(format!("    Image:    {}", c.image));
        out.push(format!("    State:    {}", c.state.word()));
        out.push(format!("    Ready:    {}", c.ready));
        out.push(format!("    Restarts: {}", c.restarts));
    }
    out.push(String::new());
    out.push("Events:".to_owned());
    out.push("  Type    Reason     Age   From               Message".to_owned());
    out.push("  ----    ------     ----  ----               -------".to_owned());
    out.push(format!(
        "  Normal  Scheduled  {}m   default-scheduler  Successfully assigned",
        pod.age.min(59),
    ));
    if !pod.phase.healthy() {
        out.push(format!(
            "  Warning BackOff    {}m   kubelet            Back-off restarting failed container",
            pod.age.min(59) / 2,
        ));
    }
    out
}

fn main() {
    let mut app = App::new();

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    driver.frame(|cx| app.ui(cx));
    let _ = app.answer();

    loop {
        driver.frame(|cx| app.ui(cx));
        // **After the draw**: a prompt's `Enter` and a dialog's answer are both written inside the
        // frame and cannot be acted on from there.
        let acted = app.answer();
        // **The window onto the frame that has just drawn**, never the one before it.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        // **A frame is owed whenever the inbox moved**, and not only when a key arrived: the
        // decision is acted on *after* the draw, so the screen showing a dialog that has just been
        // answered is one frame stale — and with nothing else pending, `wait` would park on it.
        //
        // **A focus move owes a frame too, and this loop deliberately no longer counts it.** The
        // ring resolves its walk in `settle`, *after* the draw, so the frame that consumed a `Tab`
        // painted the old focus ring with an empty `unhandled` — and this file used to compare
        // `Driver::inspect().focused()` across frames because nothing else asked. The award's own wake
        // put the ask where the decision is made: `Frame::resolve_award` calls
        // `wants_another_frame`, which is `deadline(now)`, so `wait` returns at once rather than
        // parking on a screen a keystroke behind. Comparing here as well would be a second spelling
        // of one rule, in the place least able to keep it true.
        if acted || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
