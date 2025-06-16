# Compiletime Results
### *on the whole: 3.28s → 3.34s —* $\color{red}\textsf{↑ 55.28ms (1.66\\%)}$
| test crate | old compile time | new compile time | diff | verdict |
| - | - | - | - | - |
| **"perf/vec/string"** | 186.52ms | 193.41ms | $\color{red}\textsf{↑ 6.89ms (3.56\\%)}$ | PotentialRegression { sample_std_dev: 4.321ms } |
| "perf/btreeset/insert_multi" | 342.19ms | 352.28ms | $\color{red}\textsf{↑ 10.10ms (2.87\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(9.2325ms)) |
| "perf/vec/box_dyn" | 179.05ms | 184.09ms | $\color{red}\textsf{↑ 5.04ms (2.74\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(3.875ms)) |
| "perf/misc/struct_defs" | 371.20ms | 380.93ms | $\color{red}\textsf{↑ 9.73ms (2.55\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(9.06ms)) |
| "perf/btreeset/insert_same" | 340.12ms | 347.63ms | $\color{red}\textsf{↑ 7.51ms (2.16\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(9.5915ms)) |
| "perf/vec/vec" | 388.73ms | 395.30ms | $\color{red}\textsf{↑ 6.57ms (1.66\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(8.181ms)) |
| "perf/hashset" | 503.36ms | 508.56ms | $\color{red}\textsf{↑ 5.20ms (1.02\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(13.337ms)) |
| "perf/format" | 474.88ms | 479.06ms | $\color{red}\textsf{↑ 4.19ms (0.87\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(8.553ms)) |
| "perf/misc/array_fold" | 141.26ms | 142.05ms | $\color{red}\textsf{↑ 786.17µs (0.55\\%)}$ | ProbablyNoise(SmallComparedToStdDevOf(6.3495ms)) |
| "perf/btreeset/insert_any" | 355.95ms | 355.23ms | $\color{green}\textsf{↓ 725.00µs (0.20\\%)}$ | Improved |

[^1]: thresholds: (std_dev: 1.5, absolute: 0.01).
## Failing because of 1 suspected regressions[^1]:
"perf/vec/string"
