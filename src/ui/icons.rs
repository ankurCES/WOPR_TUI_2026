pub struct Icons {
    pub missile: &'static str,
    pub radar: &'static str,
    pub threat: &'static str,
    pub base: &'static str,
    pub nuke: &'static str,
    pub shield: &'static str,
    pub comm: &'static str,
    pub warning: &'static str,
}

pub const NERD: Icons = Icons {
    missile: "\u{f0544}",  // 󰕄 nf-md-rocket_launch
    radar: "\u{f0519}",    // 󰔙 nf-md-radar
    threat: "\u{f0026}",   //  nf-fa-exclamation_triangle
    base: "\u{f0276}",     // 󰉶 nf-md-home_city
    nuke: "\u{f0599}",     // 󰖙 nf-md-nuke (radioactive)
    shield: "\u{f0510}",   // 󰔐 nf-md-shield
    comm: "\u{f0a04}",     // 󰨄 nf-md-message_text
    warning: "\u{f002d}",  //  nf-fa-bell
};

pub const ASCII: Icons = Icons {
    missile: "^",
    radar: ")",
    threat: "!",
    base: "#",
    nuke: "*",
    shield: "+",
    comm: "@",
    warning: "!",
};

pub fn icons(nerd_fonts: bool) -> &'static Icons {
    if nerd_fonts { &NERD } else { &ASCII }
}
