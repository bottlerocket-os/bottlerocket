{ callPackage }:
let
  tharPackages = rec {
    inherit tharPackages;

    signpost = callPackage ./signpost/default.nix {};
    bash = callPackage ./bash/default.nix {};
    coreutils = callPackage ./coreutils/default.nix {};
    cni = callPackage ./cni/default.nix {};
    sdk = callPackage ./sdk/default.nix {};
    docker-proxy = callPackage ./docker-proxy/default.nix {};
    readline = callPackage ./readline/default.nix {};
    libnetfilter_queue = callPackage ./libnetfilter_queue/default.nix {};
    ncurses = callPackage ./ncurses/default.nix {};
    containerd = callPackage ./containerd/default.nix {};
    conntrack-tools = callPackage ./conntrack-tools/default.nix {};
    libxcrypt = callPackage ./libxcrypt/default.nix {};
    docker-init = callPackage ./docker-init/default.nix {};
    cri-tools = callPackage ./cri-tools/default.nix {};
    golang = callPackage ./golang/default.nix {};
    libnetfilter_cthelper = callPackage ./libnetfilter_cthelper/default.nix {};
    rust = callPackage ./rust/default.nix {};
    runc = callPackage ./runc/default.nix {};
    api = callPackage ./api/default.nix {};
    libseccomp = callPackage ./libseccomp/default.nix {};
    libacl = callPackage ./libacl/default.nix {};
    libnetfilter_conntrack = callPackage ./libnetfilter_conntrack/default.nix {};
    libattr = callPackage ./libattr/default.nix {};
    docker-cli = callPackage ./docker-cli/default.nix {};
    cni-plugins = callPackage ./cni-plugins/default.nix {};
    libnftnl = callPackage ./libnftnl/default.nix {};
    util-linux = callPackage ./util-linux/default.nix {};
    aws-iam-authenticator = callPackage ./aws-iam-authenticator/default.nix {};
    systemd = callPackage ./systemd/default.nix {};
    release = callPackage ./release/default.nix {};
    grub = callPackage ./grub/default.nix {};
    filesystem = callPackage ./filesystem/default.nix {};
    libcap = callPackage ./libcap/default.nix {};
    glibc = callPackage ./glibc/default.nix {};
    strace = callPackage ./strace/default.nix {};
    kernel = callPackage ./kernel/default.nix {};
    libmnl = callPackage ./libmnl/default.nix {};
    kmod = callPackage ./kmod/default.nix {};
    libexpat = callPackage ./libexpat/default.nix {};
    libnfnetlink = callPackage ./libnfnetlink/default.nix {};
    aws-eks-ami = callPackage ./aws-eks-ami/default.nix {};
    kubernetes = callPackage ./kubernetes/default.nix {};
    ripgrep = callPackage ./ripgrep/default.nix {};
    libnetfilter_cttimeout = callPackage ./libnetfilter_cttimeout/default.nix {};
    iproute = callPackage ./iproute/default.nix {};
    socat = callPackage ./socat/default.nix {};
    docker-engine = callPackage ./docker-engine/default.nix {};
    dbus-broker = callPackage ./dbus-broker/default.nix {};
    ca-certificates = callPackage ./ca-certificates/default.nix {};
    iptables = callPackage ./iptables/default.nix {};

    # Aliases
    gcc = sdk;
    kernel-headers = kernel;
  };
in
tharPackages
