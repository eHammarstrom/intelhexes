# intelhexes

Parses intelhex

## format description

https://en.wikipedia.org/wiki/Intel_HEX

https://www.keil.com/support/docs/1584/

## intelhexes vs python-intelhex

### Performance

Measured using hyperfine on a laptop with 16GB RAM and an i7-9750H,

`hyperfine -w10 -m100 PROGRAM`

|                   | intelhexes                                       | python-intelhex                                    |
| ----------------- | ------------------------------------------------ | -------------------------------------------------- |
| Time (mean ± σ)   | `3.3 ms ± 1.1 ms [User: 2.7 ms, System: 0.7 ms]` | `40.1 ms ± 0.6 ms [User: 37.0 ms, System: 3.2 ms]` |
| Range (min … max) | `2.3 ms … 8.7 ms 962 runs`                       | `39.0 ms … 42.0 ms 100 runs`                       |

### Output

Differences: python-intelhex re-aligns addresses if a sub-16-byte data address is encountered

Snippets,

intelhexes:

```
0x008260  FC 8F FF FF 43 61 6E 27  74 20 69 6E 69 74 69 61 |....Can't initia|
0x008270  6C 69 7A 65 20 6D 75 74  65 78 2C 20 77 61 73 20 |lize mutex, was |
0x008280  4E 55 4C 4C 0D 0A 00 00  43 61 6E 27 74 20 75 6E |NULL....Can't un|
0x008290  6C 6F 63 6B 20 6D 75 74  65 78 2C 20 77 61 73 20 |lock mutex, was |
0x0082a0  4E 55 4C 4C 0D 0A 00 00  43 6F 75 6C 64 20 6E 6F |NULL....Could no|
0x0082b0  74 20 6C 6F 63 6B 20 70  6F 77 65 72 20 73 61 76 |t lock power sav|
0x0082c0  65 20 6D 75 74 65 78 00                          |e mutex.        |
0x0082c8  04 00 02 00 00 14 00 00  00 00 00 00 02 00 02 00 |................|
0x0082d8  00 0E 5C 04 05 06 07 08  01 11 00 00 25 26 27 03 |..\.........%&'.|
0x0082e8  3F 49 F6 D4 A3 C5 5F 38  74 C9 B3 E3 D2 10 3F 50 |?I...._8t.....?P|
```

python-intelhex:

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
