# intelhexes

Parses intelhex

## format description

https://en.wikipedia.org/wiki/Intel_HEX

https://www.keil.com/support/docs/1584/

## intelhexes vs python-intelhex

### Performance

Measured on a laptop with 16GB RAM and an i7-9750H,

#### Hyperfine benchmarks

`hyperfine -w10 -m100 "PROGRAM hex-examples/NINA-W15X-SW-4.0.0-006.hex > /dev/null"`

| PROGRAM           | intelhexes                                         | python-intelhex                                        |
| ----------------- | ------------------------------------------------   | --------------------------------------------------     |
| Time (mean ± σ)   | `32.7 ms ± 0.4 ms [User: 31.9 ms, System: 1.0 ms]` | `888.5 ms ± 29.8 ms [User: 861.4 ms, System: 26.8 ms]` |
| Range (min … max) | `32.1 ms … 34.0 ms 88 runs`                        | `841.5 ms … 925.6 ms 10 runs`                          |

#### Criterion benchmarks

Intelhexes throughput,

```
NRF/hex2dump/97842      time:   [825.36 us 826.62 us 828.10 us]
                        thrpt:  [112.68 MiB/s 112.88 MiB/s 113.05 MiB/s]

    Found 11 outliers among 100 measurements (11.00%)
      6 (6.00%) high mild
      5 (5.00%) high severe

NINA/hex2dump/3414628   time:   [29.971 ms 30.029 ms 30.098 ms]
                        thrpt:  [108.20 MiB/s 108.45 MiB/s 108.65 MiB/s]

    Found 7 outliers among 100 measurements (7.00%)
      3 (3.00%) high mild
      4 (4.00%) high severe
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
