//! Razer's cloud EQ library — 41 presets from Synapse.

use crate::Eq;

/// One entry in Razer's cloud EQ library.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryEntry {
    pub id: u8,
    pub name: &'static str,
    pub bands: Eq,
    pub footstep: Eq,
}

const LIBRARY: [LibraryEntry; 41] = [
    LibraryEntry { id: 0,  name: "Default",    bands: eq(0,0,0,0,0,0,0,0,0,0), footstep: eq(0,0,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 1,  name: "Game",       bands: eq(2,2,5,5,1,-1,2,3,3,3), footstep: eq(0,0,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 2,  name: "Music",      bands: eq(2,2,0,0,1,-1,-1,3,3,3), footstep: eq(0,0,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 3,  name: "Movie",      bands: eq(3,3,3,-1,-4,-4,2,3,3,3), footstep: eq(0,0,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 10, name: "Valorant",   bands: eq(1,1,-1,0,2,0,4,4,4,-3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 11, name: "Counter-Strike 2", bands: eq(-2,-1,-1,-2,3,3,0,1,2,1), footstep: eq(0,0,0,1,1,1,0,0,0,0) },
    LibraryEntry { id: 12, name: "Fortnite",   bands: eq(-4,-3,-1,3,-2,-2,4,4,4,0), footstep: eq(0,1,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 13, name: "Call of Duty", bands: eq(-2,0,3,3,3,0,0,3,3,0), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 14, name: "Apex Legends", bands: eq(2,2,3,3,-1,-2,1,4,2,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 15, name: "Halo Infinite", bands: eq(0,0,2,2,2,1,1,2,2,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 16, name: "CS2 · NiKo", bands: eq(-2,-1,-1,-2,3,3,0,1,2,1), footstep: eq(0,0,0,1,1,1,0,0,0,0) },
    LibraryEntry { id: 17, name: "Valorant · Mako · DRX", bands: eq(-1,0,1,1,-1,-1,0,0,-1,-2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 18, name: "Apex Legends · Hakis · Alliance", bands: eq(-1,0,0,0,-1,0,2,3,2,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 19, name: "Fortnite · PodaSai · Gentle Mates", bands: eq(0,0,1,1,0,0,0,1,0,0), footstep: eq(0,1,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 20, name: "CoD · Shotzzy · Optic", bands: eq(-6,-2,1,4,3,-1,0,-2,-4,-3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 21, name: "Halo · Snip3down", bands: eq(-3,-1,3,4,5,2,1,-1,-1,0), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 22, name: "CS2 · Spinx · MOUZ", bands: eq(-2,-1,-1,-2,3,3,0,1,2,1), footstep: eq(0,0,0,1,1,1,0,0,0,0) },
    LibraryEntry { id: 23, name: "Valorant · Zellsis · Sentinels", bands: eq(1,4,-1,4,2,3,4,4,4,-3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 24, name: "Apex Legends · Unlucky · Alliance", bands: eq(2,2,3,0,-1,-2,0,2,2,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 25, name: "Fortnite · Vanyak3kk · Gentle Mates", bands: eq(1,3,3,3,2,1,1,0,2,2), footstep: eq(0,1,1,1,0,0,0,0,0,0) },
    LibraryEntry { id: 26, name: "CoD · Dashy · Optic", bands: eq(-6,-2,1,4,3,-1,0,-2,-4,-3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 27, name: "Halo · LastShot · Shopify Rebellion", bands: eq(2,1,0,-2,0,2,3,4,2,0), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 28, name: "Delta Force", bands: eq(2,2,-3,-3,-3,4,4,4,4,4), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 29, name: "Battlefield 6", bands: eq(1,1,2,2,2,-1,-1,4,4,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 30, name: "CS2 · dank1ng", bands: eq(-2,-1,-1,-2,3,4,0,1,2,1), footstep: eq(0,0,0,1,1,1,0,0,0,0) },
    LibraryEntry { id: 31, name: "Valorant · sword9 · TYLOO", bands: eq(2,2,0,0,1,-1,-1,3,3,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 32, name: "Valorant · slowly · TYLOO", bands: eq(1,1,-1,0,2,0,4,4,4,-5), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 33, name: "Valorant · Splash · TYLOO", bands: eq(2,2,5,5,1,-1,2,3,3,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 34, name: "Valorant · Scales · TYLOO", bands: eq(1,2,1,2,3,1,4,3,1,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 35, name: "Valorant · ERV · TYLOO", bands: eq(2,2,0,0,1,-1,-1,3,3,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 36, name: "Valorant · cortezia · Sentinels", bands: eq(2,2,4,4,1,0,1,3,3,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 37, name: "Valorant · johnqt · Sentinels", bands: eq(2,2,5,5,1,-1,2,3,3,3), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 38, name: "Valorant · Reduxx · Sentinels", bands: eq(1,1,1,1,1,1,1,1,1,1), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 39, name: "Delta Force · BanJing · Jaguar Ace Gaming", bands: eq(1,-1,1,2,2,0,3,2,3,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 40, name: "Delta Force · GaoXin · Jaguar Ace Gaming", bands: eq(1,-1,1,2,2,0,3,2,3,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 41, name: "Delta Force · YERAN · Jaguar Ace Gaming", bands: eq(1,-1,1,2,2,0,3,2,3,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 42, name: "Delta Force · ZGOD · Jaguar Ace Gaming", bands: eq(1,-1,1,2,2,0,3,2,3,2), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 43, name: "Delta Force · Blank · TE 溯", bands: eq(-2,-2,0,1,2,0,0,1,-1,-1), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 44, name: "Delta Force · GuiRoW · TE 溯", bands: eq(-2,-2,0,1,2,0,0,1,-1,-1), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 45, name: "Delta Force · Kellyt · TE 溯", bands: eq(-2,-2,0,1,2,0,0,1,-1,-1), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
    LibraryEntry { id: 46, name: "Delta Force · Moxi · TE 溯", bands: eq(-2,-2,0,1,2,0,0,1,-1,-1), footstep: eq(0,0,1,1,1,0,0,0,0,0) },
];

/// Helper: const fn to build an EQ from 10 i8 values.
const fn eq(a: i8, b: i8, c: i8, d: i8, e: i8, f: i8, g: i8, h: i8, i: i8, j: i8) -> Eq {
    Eq([a, b, c, d, e, f, g, h, i, j])
}

/// Every preset seen in the Synapse capture, ordered by id.
pub fn library() -> Vec<LibraryEntry> {
    LIBRARY.to_vec()
}

/// Look up a library entry by cloud EQ ID.
pub fn library_entry_by_id(id: u8) -> Option<LibraryEntry> {
    LIBRARY.iter().find(|e| e.id == id).cloned()
}

/// Look up a library entry name by cloud EQ ID.
pub fn library_name(id: u8) -> Option<&'static str> {
    LIBRARY.iter().find(|e| e.id == id).map(|e| e.name)
}

/// Find a library entry by its Synapse name (case-sensitive).
pub fn library_by_name(name: &str) -> Option<LibraryEntry> {
    LIBRARY.iter().find(|e| e.name == name).cloned()
}
