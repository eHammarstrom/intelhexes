# intelhexes

Intelhexes implements the following listed features, as found in
[python-intelhex](https://github.com/python-intelhex/intelhex), while
performing the operations faster.

- [x] hex2dump
- [ ] bin2hex
- [x] hex2bin

## intelhexes vs python-intelhex

### Performance

Measured on a laptop with 16GB RAM and an i7-10750H,

#### Hyperfine hex2dump benchmarks

`hyperfine -w10 -m100 "PROGRAM hex-examples/NINA-W15X-SW-4.0.0-006.hex > /dev/null"`

| PROGRAM           | intelhexes                                         | python-intelhex                                        |
| ----------------- | ------------------------------------------------   | --------------------------------------------------     |
| Time (mean ± σ)   | `27.5 ms ± 1.1 ms [User: 27.2 ms, System: 0.8 ms]` | `1.117 s ± 0.024 s [User: 1.071 s, System: 0.046 s]` |
| Range (min … max) | `26.9 ms …  32.7 ms 103 runs`                        | `1.082 s …  1.205 s 100 runs` |

#### Criterion benchmarks

hex2dump,
```

NRF/hex2dump/97842      time:   [713.76 µs 715.13 µs 716.54 µs]
                        thrpt:  [130.22 MiB/s 130.48 MiB/s 130.73 MiB/s]
                        change:
Found 7 outliers among 100 measurements (7.00%)
5 (5.00%) high mild
2 (2.00%) high severe

NINA/hex2dump/3414628   time:   [23.997 ms 24.204 ms 24.412 ms]
                        thrpt:  [133.39 MiB/s 134.54 MiB/s 135.70 MiB/s]
```

hex2bin,
```
NRF/hex2bin/97842       time:   [335.50 µs 336.29 µs 337.37 µs]
                        thrpt:  [276.58 MiB/s 277.46 MiB/s 278.12 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

NINA/hex2bin/3414628    time:   [12.331 ms 12.492 ms 12.660 ms]
                        thrpt:  [257.23 MiB/s 260.68 MiB/s 264.09 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
```

### Output versus python-intelhex

intelhexes (hex2dump):

```
0x00008260  FC 8F FF FF 43 61 6E 27  74 20 69 6E 69 74 69 61  |....Can't initia|
0x00008270  6C 69 7A 65 20 6D 75 74  65 78 2C 20 77 61 73 20  |lize mutex, was |
0x00008280  4E 55 4C 4C 0D 0A 00 00  43 61 6E 27 74 20 75 6E  |NULL....Can't un|
0x00008290  6C 6F 63 6B 20 6D 75 74  65 78 2C 20 77 61 73 20  |lock mutex, was |
0x000082A0  4E 55 4C 4C 0D 0A 00 00  43 6F 75 6C 64 20 6E 6F  |NULL....Could no|
0x000082B0  74 20 6C 6F 63 6B 20 70  6F 77 65 72 20 73 61 76  |t lock power sav|
0x000082C0  65 20 6D 75 74 65 78 00  04 00 02 00 00 14 00 00  |e mutex.........|
0x000082D0  00 00 00 00 02 00 02 00  00 0E 5C 04 05 06 07 08  |..........\.....|
0x000082E0  01 11 00 00 25 26 27 03  3F 49 F6 D4 A3 C5 5F 38  |....%&'.?I...._8|
0x000082F0  74 C9 B3 E3 D2 10 3F 50  4A FF 60 7B EB 40 B7 99  |t.....?PJ.`{.@..|
```

python-intelhex (hex2dump.py):

```
8260  FC 8F FF FF 43 61 6E 27 74 20 69 6E 69 74 69 61  |....Can't initia|
8270  6C 69 7A 65 20 6D 75 74 65 78 2C 20 77 61 73 20  |lize mutex, was |
8280  4E 55 4C 4C 0D 0A 00 00 43 61 6E 27 74 20 75 6E  |NULL....Can't un|
8290  6C 6F 63 6B 20 6D 75 74 65 78 2C 20 77 61 73 20  |lock mutex, was |
82A0  4E 55 4C 4C 0D 0A 00 00 43 6F 75 6C 64 20 6E 6F  |NULL....Could no|
82B0  74 20 6C 6F 63 6B 20 70 6F 77 65 72 20 73 61 76  |t lock power sav|
82C0  65 20 6D 75 74 65 78 00 04 00 02 00 00 14 00 00  |e mutex.........|
82D0  00 00 00 00 02 00 02 00 00 0E 5C 04 05 06 07 08  |..........\.....|
82E0  01 11 00 00 25 26 27 03 3F 49 F6 D4 A3 C5 5F 38  |....%&'.?I...._8|
82F0  74 C9 B3 E3 D2 10 3F 50 4A FF 60 7B EB 40 B7 99  |t.....?PJ.`{.@..|
```

## format description

https://en.wikipedia.org/wiki/Intel_HEX

https://www.keil.com/support/docs/1584/
