# Allocation results

Five independent release processes were run for every `(mode, operation)` pair at 20,000 operations
per process. The six combinations were rotated between repetitions. Source was clean at
`91a31104e4b5c099b32ea6b9cf1abfa0964b4eda`.

## Medians

| operation | mode | gross bytes | allocations | bytes vs off | allocations vs off |
| --- | --- | ---: | ---: | ---: | ---: |
| insert | off | 6,156,432 | 142,060 | baseline | baseline |
| insert | listener | 13,613,472 | 208,450 | +121.13% | +46.73% |
| insert | observer | 6,158,480 | 142,060 | +0.03% | 0.00% |
| remove | off | 45,106,032 | 23,724 | baseline | baseline |
| remove | listener | 58,463,936 | 209,870 | +29.61% | +784.63% |
| remove | observer | 45,108,272 | 23,732 | +0.01% | +0.03% |

The insert case measures the fixed cost of enabling notification without removing an entry in the
measured loop. The remove case includes callback delivery. Gross allocated bytes count allocation
traffic, not retained memory. These data support an allocation-ownership claim only; they do not
establish an elapsed-time speedup.

## Integrity

- `raw.csv`: `1734e133d97073322999223bc465456dcaa5db1ac4753ffdb19ba1679fcd7511`
- `metadata.txt`: `313f7f4e5707d87c2a96d30ba0fd476bc7e1db1d47c68930c414fe530340a6a9`
