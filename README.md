# S.T.A.R.
(Simple Time-sharing system And Resource manager)

A monolithic Kernel/OS implementation for RISC-V. Targets 64 Bit.


## Run Locally

Clone the project
```bash
  git clone https://github.com/laniku/star.git
```
Go to the project directory
```bash
  cd star
```
Get OpenSBI (QEMU will fail if not downloaded)

```bash
  curl -LO https://github.com/qemu/qemu/raw/v8.0.4/pc-bios/opensbi-riscv32-generic-fw_dynamic.bin
```

Build & Run

```bash
  ./run.sh
```

## Acknowledgements

 - [Hypervisor in 1,000 Lines (for some base work)](https://1000hv.seiya.me/en/)
 - [RISC-V Instruction Set Specifications by @msyksphinz-self](https://msyksphinz-self.github.io/riscv-isadoc/)
