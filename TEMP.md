Это отличное, технически грамотное описание. Оно полностью соответствует стандартам индустрии (Agilent, Waters, IUPAC).

Если вы хотите интегрировать это в общее описание набора данных, вот финальный отшлифованный вариант на английском и русском:

### English
**Selectivity Factor (Alpha, $\alpha$)**

The **Selectivity Factor** (also known as the separation factor or relative retention) measures the thermodynamic separation between two peaks. It is used to determine how far apart the retention factors ($k'$) of two components are and whether a separation can be achieved.

$$\alpha = \frac{k'_{i + 1}}{k'_i}$$

*   **Condition**: $k'_{i + 1} > k'_i$, therefore $\alpha > 1$ is required for separation to take place.
*   **Retention Factor ($k'$)**: Calculated as $k' = (Rt - Dt) / Dt$, where $Rt$ is the retention time and $Dt$ is the dead time.

---

### Русский
**Фактор селективности (Альфа, $\alpha$)**

**Фактор селективности** (также называемый коэффициентом разделения или относительным удерживанием) характеризует термодинамическое разделение двух пиков. Он используется для оценки разности факторов удерживания ($k'$) двух компонентов и определения возможности их разделения.

$$\alpha = \frac{k'_{i + 1}}{k'_i}$$

*   **Условие**: $k'_{i + 1} > k'_i$, следовательно, для осуществления разделения необходимо значение $\alpha > 1$.
*   **Фактор удерживания ($k'$)**: Рассчитывается как $k' = (Rt - Dt) / Dt$, где $Rt$ — время удерживания, а $Dt$ — мёртвое время.

---

### Что можно улучшить в вашем тексте:
1.  **Терминология**: В современной литературе термин *capacity factor* (емкость колонки) считается устаревшим, IUPAC рекомендует использовать **retention factor** (фактор удерживания). Но так как Agilent всё еще указывает его в скобках, оставить его можно для полноты картины.
2.  **Связь с данными**: Если в вашем наборе данных значения $\alpha$ уже рассчитаны и лежат в отдельных таблицах, добавьте фразу: 
    *   *English*: "The calculated $\alpha$ values for all peak pairs are provided in the supplementary tables."
    *   *Русский*: "Рассчитанные значения $\alpha$ для всех пар пиков приведены в дополнительных таблицах."