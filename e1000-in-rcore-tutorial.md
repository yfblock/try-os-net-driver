# Using e1000 with rCore Tutorial v3

## using pci to scan e1000 driver

#### Refs:

[https://github.com/rcore-os/pci-rs](https://github.com/rcore-os/pci-rs)
[https://github.com/rcore-os/isomorphic_drivers](https://github.com/rcore-os/isomorphic_drivers)

#### Add memory map for kernel space.

```rust
println!("mapping pci memory");
memory_set.push(
    MapArea::new(
        0x2fffffff.into(),
        0x3fffffff.into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
    ),
    None,
);
println!("mapping nvme memory");
memory_set.push(
    MapArea::new(
        0x3fffffff.into(),
        0x4000ffff.into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
    ),
    None,
);
```

#### add pci mod to source code.

```plain
pci
├── mod.rs
└── pci_impl.rs
```

#### add init in mod.rs


``` rust
pub fn init() {
    const BAR_LEN: usize = 40;
    println!("{:-^1$}", "PCI INIT", BAR_LEN);
    pci_scan();
    println!("{:-^1$}", "PCI INIT SUCCESS", BAR_LEN);
}
```
#### add drivers to Cargo.toml

```toml
isomorphic_drivers = { git = "https://github.com/rcore-os/isomorphic_drivers", rev = "f7cd97a8", features = [
    "log",
] }
```
