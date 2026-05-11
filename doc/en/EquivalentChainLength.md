# Equivalent chain length

$$ECL(x) = n \cdot \frac{\log t'_R(x) − \log t'_R(z)}{\log t'_R(z + n) − \log t'_R(z)} + z \tag{1}$$

$$ECL(x) = n \cdot \frac{t_R(x) − t_R(z)}{t_R(z + n) − t_R(z)} + z \tag{2}$$

---

$$ECL(x) = C(F) + (C(B) - C(F)) \times \frac{t_R(x) − t_R(F)}{t_R(B) − t_R(F)} \tag{2}$$

**где**:

- $t_R$ и $t'_R$ — абсолютные и исправленные времена удерживания соответственно интересующего соединения и двух эталонов (насыщенных FAME), элюирующихся по обе стороны от искомого соединения;
- $z$ — представляет собой число атомов углерода в эталоне, элюирующемся перед $x$;
- $n$ — разница в атомах углерода между двумя эталонами.

- $t_R(F)$ — абсолютное время удерживания в эталоне, элюирующемся перед $x$;
- $t_R(B)$ — в эталоне, элюирующемся перед $x$;
- $C(F)$ — число атомов углерода в эталоне, элюирующемся перед $x$;
- $C(B)$ — число атомов углерода в эталоне, элюирующемся после $x$;

> [!NOTE]
> Для максимальной точности рекомендуется, чтобы $n$ было равно единице.

## Abbreviations

- ECL

## Description

Two compounds can be separated if they have sufficiently different retention factors – $k'$ values.

**Физический смысл**:

Фактор удерживания $k'$ показывает, во сколько раз дольше вещество взаимодействует с неподвижной фазой по сравнению с подвижной. Если $k = 3$, это значит, что молекула взаимодействует с сорбентом в 3 раза больше времени, чем с потоком растворителя. Это безразмерная величина, которая не зависит от скорости потока или длины колонки, а зависит только от химической природы сорбента, растворителя и самого вещества.

- **$t_R$ (Retention time)** — общее время удерживания. Это время от момента ввода пробы до выхода максимума (вершины) пика целевого вещества.
- **$t_0$ (Dead time)** — мертвое время. Это время, за которое через колонку проходит неудерживаемый компонент (молекула, которая вообще не взаимодействует с неподвижной фазой).
- **$t'_R = t_R - t_0$** — исправленное время удерживания. Это чистое время, которое молекула аналита взаимодействует с неподвижной фазой (сорбентом).

## References

1. - [Miwa et al., 1960](https://doi.org/10.1021%2Fac50153a010 "Gas chromatographic characterization of fatty acids.Identification constants for mono- and dicarboxylic methyl esters")
   - [Woodford et al., 1960](https://doi.org/10.1016/s0022-2275(20)39082-9 "Gas-liquid chromatography of fatty acid methyl esters: the “carbon-number” as a parameter for comparison of columns")
   - [Mjøs, 2006](https://doi.org/10.1016/j.chroma.2006.04.067 "Prediction of equivalent chain lengths from two-dimensional fatty acid retention indices")
2. - [van Den Dool et al., 1963](https://doi.org/10.1016/S0021-9673(01)80947-X "A generalization of the retention index system including linear temperature programmed gas—liquid partition chromatography")
   - [Mjøs, 2006](https://doi.org/10.1016/j.chroma.2006.04.067 "Prediction of equivalent chain lengths from two-dimensional fatty acid retention indices")
