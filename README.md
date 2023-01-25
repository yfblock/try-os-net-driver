# e1000 driver

[https://github.com/LearningOS/2023-undergraduate--graduation-design/issues/8](https://github.com/LearningOS/2023-undergraduate--graduation-design/issues/8)

## Reference

[https://wiki.osdev.org/Intel_8254x](https://wiki.osdev.org/Intel_8254x)

[https://pdos.csail.mit.edu/6.828/2017/labs/lab6/](https://pdos.csail.mit.edu/6.828/2017/labs/lab6/)

[https://pdos.csail.mit.edu/6.828/2022/labs/net.html](https://pdos.csail.mit.edu/6.828/2022/labs/net.html)

[https://pdos.csail.mit.edu/6.828/2022/readings/8254x_GBe_SDM.pdf](https://pdos.csail.mit.edu/6.828/2022/readings/8254x_GBe_SDM.pdf)

## e1000 driver

### initialize

set CTRL.RST (0x4000000) of CTRL (0x00000) register to reset device.

```c
// Reset device but keep the PCI config
e1000_reg_write(E1000_CNTRL_REG, e1000_reg_read(E1000_CNTRL_REG, the_e1000) | E1000_CNTRL_RST_MASK, the_e1000);
//read back the value after approx 1us to check RST bit cleared
```

Waiting for the reset done.
```c
//read back the value after approx 1us to check RST bit cleared
do {
udelay(3);
}while(E1000_CNTRL_RST_BIT(e1000_reg_read(E1000_CNTRL_REG, the_e1000)));
```

Before reading the EEPROM has a "lock-unlock" mechanism to prevent software-hardware collisions when reading from the EEPROM.

ASDE(Auto-Speed Detection Enable).
SLU(Set Link up).

The "Set Link Up" is normally initialized to 0b. However, if either the APM Enable or SMBus Enable bits are set in the EEPROM then it is initialized to 1b, ensuring MAC/PHY communication during preboot states.

```c
//the manual says in Section 14.3 General Config -
//Must set the ASDE and SLU(bit 5 and 6(0 based index)) in the CNTRL Reg to allow auto speed
//detection after RESET
uint32_t cntrl_reg = e1000_reg_read(E1000_CNTRL_REG, the_e1000);
e1000_reg_write(E1000_CNTRL_REG, cntrl_reg | E1000_CNTRL_ASDE_MASK | E1000_CNTRL_SLU_MASK, the_e1000);
```

After that the EEPROM must be enabled in order to be able to read the MAC address of the NIC, this is done by setting the EECD.SK (0x01), EECD.CS (0x02) and EECD.DI (0x04) bits of the EECD (0x00010) register. This will allow software to perform reads to the EEPROM.

```c
//Read Hardware(MAC) address from the device
uint32_t macaddr_l = e1000_reg_read(E1000_RCV_RAL0, the_e1000);
uint32_t macaddr_h = e1000_reg_read(E1000_RCV_RAH0, the_e1000);
*(uint32_t*)the_e1000->mac_addr = macaddr_l;
*(uint16_t*)(&the_e1000->mac_addr[4]) = (uint16_t)macaddr_h;
*(uint32_t*)mac_addr = macaddr_l;
*(uint32_t*)(&mac_addr[4]) = (uint16_t)macaddr_h;
char mac_str[18];
unpack_mac(the_e1000->mac_addr, mac_str);
mac_str[17] = 0;
```

Init receive and transmit

```c
//Write the Descriptor ring addresses in TDBAL, and RDBAL, plus HEAD and TAIL pointers
e1000_reg_write(E1000_TDBAL, V2P(the_e1000->tbd[0]), the_e1000);
e1000_reg_write(E1000_TDBAH, 0x00000000, the_e1000);
e1000_reg_write(E1000_TDLEN, (E1000_TBD_SLOTS*16) << 7, the_e1000);
e1000_reg_write(E1000_TDH, 0x00000000, the_e1000);
e1000_reg_write(E1000_TCTL,
                E1000_TCTL_EN |
                E1000_TCTL_PSP |
                E1000_TCTL_CT_SET(0x0f) |
                E1000_TCTL_COLD_SET(0x200),
                the_e1000);
e1000_reg_write(E1000_TDT, 0, the_e1000);

e1000_reg_write(E1000_TIPG,
                E1000_TIPG_IPGT_SET(10) |
                E1000_TIPG_IPGR1_SET(10) |
                E1000_TIPG_IPGR2_SET(10),
                the_e1000);
e1000_reg_write(E1000_RDBAL, V2P(the_e1000->rbd[0]), the_e1000);
e1000_reg_write(E1000_RDBAH, 0x00000000, the_e1000);
e1000_reg_write(E1000_RDLEN, (E1000_RBD_SLOTS*16) << 7, the_e1000);
e1000_reg_write(E1000_RDH, 0x00000000, the_e1000);
e1000_reg_write(E1000_RDT, 0x00000000, the_e1000);
```

Enable Interrupt.


```c
//enable interrupts
e1000_reg_write(E1000_IMS, E1000_IMS_RXSEQ | E1000_IMS_RXO | E1000_IMS_RXT0|E1000_IMS_TXQE, the_e1000);
//Receive control Register.
e1000_reg_write(E1000_RCTL,
            E1000_RCTL_EN |
                E1000_RCTL_BAM |
                E1000_RCTL_BSIZE | 0x00000008,//|
            //  E1000_RCTL_SECRC,
            the_e1000);
```

### Transmit

```c
struct e1000 *e1000 = (struct e1000*)driver;
cprintf("e1000 driver: Sending packet of length:0x%x %x starting at physical address:0x%x\n", length, sizeof(struct ethr_hdr), V2P(e1000->tx_buf[e1000->tbd_tail]));
memset(e1000->tbd[e1000->tbd_tail], 0, sizeof(struct e1000_tbd));
memmove((e1000->tx_buf[e1000->tbd_tail]), pkt, length);
e1000->tbd[e1000->tbd_tail]->addr = (uint64_t)(uint32_t)V2P(e1000->tx_buf[e1000->tbd_tail]);
e1000->tbd[e1000->tbd_tail]->length = length;
e1000->tbd[e1000->tbd_tail]->cmd = (E1000_TDESC_CMD_RS | E1000_TDESC_CMD_EOP | E1000_TDESC_CMD_IFCS);
e1000->tbd[e1000->tbd_tail]->cso = 0;
// update the tail so the hardware knows it's ready
int oldtail = e1000->tbd_tail;
e1000->tbd_tail = (e1000->tbd_tail + 1) % E1000_TBD_SLOTS;
e1000_reg_write(E1000_TDT, e1000->tbd_tail, e1000);

while( !E1000_TDESC_STATUS_DONE(e1000->tbd[oldtail]->status) )
{
    udelay(2);
}
cprintf("after while loop\n");
```

### Receive

> No implementation.




## e1000 driver of xv6-net

The xv6-net-net folder descript it more clearly.

```c
int
e1000_init(struct pci_func *pcif)
{
    pci_func_enable(pcif);
    struct e1000 *dev = (struct e1000 *)kalloc();
    // Resolve MMIO base address
    dev->mmio_base = e1000_resolve_mmio_base(pcif);
    assert(dev->mmio_base);
    cprintf("[e1000] mmio_base=0x%08x\n", dev->mmio_base);
    // Read HW address from EEPROM
    e1000_read_addr_from_eeprom(dev, dev->addr);
    cprintf("[e1000] addr=%02x:%02x:%02x:%02x:%02x:%02x\n", dev->addr[0], dev->addr[1], dev->addr[2], dev->addr[3], dev->addr[4], dev->addr[5]);
    // Register I/O APIC
    dev->irq = pcif->irq_line;
    ioapicenable(dev->irq, ncpu - 1);
    // Initialize Multicast Table Array
    for (int n = 0; n < 128; n++)
        e1000_reg_write(dev, E1000_MTA + (n << 2), 0);
    // Initialize RX/TX
    e1000_rx_init(dev);
    e1000_tx_init(dev);
    // Alloc netdev
    struct netdev *netdev = netdev_alloc(e1000_setup);
    memcpy(netdev->addr, dev->addr, 6);
    netdev->priv = dev;
    netdev->ops = &e1000_ops;
    netdev->flags |= NETDEV_FLAG_RUNNING;
    // Register netdev
    netdev_register(netdev);
    dev->netdev = netdev;
    // Link to e1000 device list
    dev->next = devices;
    devices = dev;
    return 0;
}
```

Initialize Rx

```c
// initialize rx descriptors
for(int n = 0; n < RX_RING_SIZE; n++) {
    memset(&dev->rx_ring[n], 0, sizeof(struct rx_desc));
    // alloc DMA buffer
    dev->rx_ring[n].addr = (uint64_t)V2P(kalloc());
}
// setup rx descriptors
uint64_t base = (uint64_t)(V2P(dev->rx_ring));
e1000_reg_write(dev, E1000_RDBAL, (uint32_t)(base & 0xffffffff));
e1000_reg_write(dev, E1000_RDBAH, (uint32_t)(base >> 32));
// rx descriptor lengh
e1000_reg_write(dev, E1000_RDLEN, (uint32_t)(RX_RING_SIZE * sizeof(struct rx_desc)));
// setup head/tail
e1000_reg_write(dev, E1000_RDH, 0);
e1000_reg_write(dev, E1000_RDT, RX_RING_SIZE-1);
// set tx control register
e1000_reg_write(dev, E1000_RCTL, (
    E1000_RCTL_SBP        | /* store bad packet */
    E1000_RCTL_UPE        | /* unicast promiscuous enable */
    E1000_RCTL_MPE        | /* multicast promiscuous enab */
    E1000_RCTL_RDMTS_HALF | /* rx desc min threshold size */
    E1000_RCTL_SECRC      | /* Strip Ethernet CRC */
    E1000_RCTL_LPE        | /* long packet enable */
    E1000_RCTL_BAM        | /* broadcast enable */
    E1000_RCTL_SZ_2048    | /* rx buffer size 2048 */
    0)
);
```

Initialize TX

```c
// initialize tx descriptors
for (int n = 0; n < TX_RING_SIZE; n++) {
    memset(&dev->tx_ring[n], 0, sizeof(struct tx_desc));
}
// setup tx descriptors
uint64_t base = (uint64_t)(V2P(dev->tx_ring));
e1000_reg_write(dev, E1000_TDBAL, (uint32_t)(base & 0xffffffff));
e1000_reg_write(dev, E1000_TDBAH, (uint32_t)(base >> 32) );
// tx descriptor length
e1000_reg_write(dev, E1000_TDLEN, (uint32_t)(TX_RING_SIZE * sizeof(struct tx_desc)));
// setup head/tail
e1000_reg_write(dev, E1000_TDH, 0);
e1000_reg_write(dev, E1000_TDT, 0);
// set tx control register
e1000_reg_write(dev, E1000_TCTL, (
    E1000_TCTL_PSP | /* pad short packets */
    0)
);
```

Send data

```c
static ssize_t
e1000_tx_cb(struct netdev *netdev, uint8_t *data, size_t len)
{
    struct e1000 *dev = (struct e1000 *)netdev->priv;
    uint32_t tail = e1000_reg_read(dev, E1000_TDT);
    struct tx_desc *desc = &dev->tx_ring[tail];

    desc->addr = (uint64_t)V2P(data);
    desc->length = len;
    desc->status = 0;
    desc->cmd = (E1000_TXD_CMD_EOP | E1000_TXD_CMD_RS);
#ifdef DEBUG
    cprintf("[e1000] %s: %u bytes data transmit\n", dev->netdev->name, desc->length);
#endif
    e1000_reg_write(dev, E1000_TDT, (tail + 1) % TX_RING_SIZE);
    while(!(desc->status & 0x0f)) {
        microdelay(1);
    }
    return len;
}
```

Receive Data

```c
static void
e1000_rx(struct e1000 *dev)
{
#ifdef DEBUG
    cprintf("[e1000] %s: check rx descriptors...\n", dev->netdev->name);
#endif
    while (1) {
        uint32_t tail = (e1000_reg_read(dev, E1000_RDT)+1) % RX_RING_SIZE;
        struct rx_desc *desc = &dev->rx_ring[tail];
        if (!(desc->status & E1000_RXD_STAT_DD)) {
            /* EMPTY */
            break;
        }
        do {
            if (desc->length < 60) {
                cprintf("[e1000] short packet (%d bytes)\n", desc->length);
                break;
            }
            if (!(desc->status & E1000_RXD_STAT_EOP)) {
                cprintf("[e1000] not EOP! this driver does not support packet that do not fit in one buffer\n");
                break;
            }
            if (desc->errors) {
                cprintf("[e1000] rx errors (0x%x)\n", desc->errors);
                break;
            }
#ifdef DEBUG
            cprintf("[e1000] %s: %u bytes data received\n", dev->netdev->name, desc->length);
#endif
            ethernet_rx_helper(dev->netdev, P2V((uint32_t)desc->addr), desc->length, netdev_receive);
        } while (0);
        desc->status = (uint16_t)(0);
        e1000_reg_write(dev, E1000_RDT, tail);
    }
}

```

### Build & Run

```
$ make docker-build
$ make docker-run

...(xv6-net starts on qemu in the container)...

$ ifconfig net1 172.16.100.2 netmask 255.255.255.0
$ ifconfig net1 up
$ tcpechoserver
Starting TCP Echo Server
socket: success, soc=3
bind: success, self=0.0.0.0:7
waiting for connection...

...(client connection information and received data are output)...

(switch to qemu monitor with Ctrl-a + c and exit by typing `quit`)
```

*Ping Test (at another terminal)*
```
$ docker exec -it xv6-net ping 172.16.100.2
```

*TCP Test (at another terminal)*
```
$ docker exec -it xv6-net nc 172.16.100.2 7
```