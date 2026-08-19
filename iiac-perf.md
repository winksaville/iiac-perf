Set the frequency information for these benchmarks
```toml
[freq]
governor = "powersave"
epp = "balance_performance"
boost = true
# min_mhz / max_mhz omitted: the hardware range (563-4673 MHz)
# pin_mhz omitted: the base clock (3801 MHz from acpi_cppc/nominal_freq)
```

