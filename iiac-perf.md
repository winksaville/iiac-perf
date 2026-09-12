Replicate every run, so a number measured here carries an error bar without a flag being typed.
A hundred blocks keeps a five-second run's time axis at about 50 ms a block, the size the grades
were tuned on, for about 0.7 s more than ten blocks cost. The sleep is a range so each block
re-rolls scheduler and frequency state, and the warmup keeps each post-wake ramp out of the
samples. These keys sit above the `[freq]` fence because the fences concatenate in document order,
and a bare key after a table header would be parsed into that table.
```toml
blocks = 100
block_sleep = "1-10ms"
block_warmup = "2ms"
```

Set the frequency information for these benchmarks
```toml
[freq]
governor = "powersave"
epp = "balance_performance"
boost = true
# min_mhz / max_mhz omitted: the hardware range (563-4673 MHz)
# pin_mhz omitted: the base clock (3801 MHz from acpi_cppc/nominal_freq)
```

