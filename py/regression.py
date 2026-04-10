import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from scipy.stats import pearsonr
import statsmodels.formula.api as smf
import io
import numpy as np

# 1. Подготовка данных
data = """A,B,C,D
8,23.365,0.249,85.229
10,39.027,0.416,84.852
11,47.48,0.506,84.873
12,55.897,0.596,85.01
13,64.088,0.683,85.18
14,72,0.767,85.382
15,79.579,0.848,85.546
16,86.887,0.926,85.774
17,93.82,1,85.907
18,100.534,1.072,86.096
19,106.864,1.14,86.236
20,113.104,1.206,86.418
21,118.977,1.268,86.476
22,124.754,1.33,86.684
23,130.19,1.388,86.703
"""
df = pd.read_csv(io.StringIO(data))

print("Исходные данные:")
print(df.head())
print("\n" + "="*50 + "\n")

# Настройка стиля графиков
sns.set_style("whitegrid")

# --- 1. Графики рассеяния ---
print("--- 1. Графики рассеяния ---")
plt.figure(figsize=(12, 5))

plt.subplot(1, 2, 1)
sns.scatterplot(x='A', y='B', data=df)
plt.title('График рассеяния B от A')
plt.xlabel('A')
plt.ylabel('B')

plt.subplot(1, 2, 2)
sns.scatterplot(x='A', y='C', data=df, color='orange')
plt.title('График рассеяния C от A')
plt.xlabel('A')
plt.ylabel('C')

plt.tight_layout()
plt.show()
print("\n" + "="*50 + "\n")

# --- 2. Коэффициент корреляции Пирсона ---
print("--- 2. Коэффициент корреляции Пирсона ---")
corr_ab, p_ab = pearsonr(df['A'], df['B'])
corr_ac, p_ac = pearsonr(df['A'], df['C'])

print(f"Коэффициент корреляции Пирсона между A и B: {corr_ab:.4f} (p-value: {p_ab:.4f})")
print(f"Коэффициент корреляции Пирсона между A и C: {corr_ac:.4f} (p-value: {p_ac:.4f})")
print("\n" + "="*50 + "\n")

# --- 3. Линейная регрессия ---
print("--- 3. Линейная регрессия ---")

# Модель для B от A
model_b_linear = smf.ols('B ~ A', data=df).fit()
print("Линейная регрессия: B от A")
print(model_b_linear.summary())

# Модель для C от A
model_c_linear = smf.ols('C ~ A', data=df).fit()
print("\nЛинейная регрессия: C от A")
print(model_c_linear.summary())

# Графики остатков для линейной регрессии
plt.figure(figsize=(12, 5))

plt.subplot(1, 2, 1)
sns.scatterplot(x=model_b_linear.predict(), y=model_b_linear.resid)
plt.axhline(y=0, color='r', linestyle='--')
plt.title('Остатки B от A (линейная регрессия)')
plt.xlabel('Предсказанные значения B')
plt.ylabel('Остатки')

plt.subplot(1, 2, 2)
sns.scatterplot(x=model_c_linear.predict(), y=model_c_linear.resid, color='orange')
plt.axhline(y=0, color='r', linestyle='--')
plt.title('Остатки C от A (линейная регрессия)')
plt.xlabel('Предсказанные значения C')
plt.ylabel('Остатки')

plt.tight_layout()
plt.show()
print("\n" + "="*50 + "\n")

# --- 4. Полиномиальная регрессия (степень 2) ---
print("--- 4. Полиномиальная регрессия (степень 2) ---")

# Модель для B от A (квадратичная)
model_b_poly = smf.ols('B ~ A + I(A**2)', data=df).fit()
print("Полиномиальная регрессия (степень 2): B от A")
print(model_b_poly.summary())

# Модель для C от A (квадратичная)
model_c_poly = smf.ols('C ~ A + I(A**2)', data=df).fit()
print("\nПолиномиальная регрессия (степень 2): C от A")
print(model_c_poly.summary())

# Графики остатков для полиномиальной регрессии
plt.figure(figsize=(12, 5))

plt.subplot(1, 2, 1)
sns.scatterplot(x=model_b_poly.predict(), y=model_b_poly.resid)
plt.axhline(y=0, color='r', linestyle='--')
plt.title('Остатки B от A (полиномиальная регрессия)')
plt.xlabel('Предсказанные значения B')
plt.ylabel('Остатки')

plt.subplot(1, 2, 2)
sns.scatterplot(x=model_c_poly.predict(), y=model_c_poly.resid, color='orange')
plt.axhline(y=0, color='r', linestyle='--')
plt.title('Остатки C от A (полиномиальная регрессия)')
plt.xlabel('Предсказанные значения C')
plt.ylabel('Остатки')

plt.tight_layout()
plt.show()

# Визуализация полиномиальной подгонки на графиках рассеяния
plt.figure(figsize=(12, 5))

# Для B от A
plt.subplot(1, 2, 1)
sns.scatterplot(x='A', y='B', data=df, label='Данные')
# Создаем точки для линии подгонки
A_range = np.linspace(df['A'].min(), df['A'].max(), 100)
df_pred_b = pd.DataFrame({'A': A_range})
plt.plot(A_range, model_b_poly.predict(df_pred_b), color='red', linestyle='--', label='Полиномиальная подгонка')
plt.title('B от A с полиномиальной регрессией')
plt.xlabel('A')
plt.ylabel('B')
plt.legend()

# Для C от A
plt.subplot(1, 2, 2)
sns.scatterplot(x='A', y='C', data=df, color='orange', label='Данные')
df_pred_c = pd.DataFrame({'A': A_range})
plt.plot(A_range, model_c_poly.predict(df_pred_c), color='red', linestyle='--', label='Полиномиальная подгонка')
plt.title('C от A с полиномиальной регрессией')
plt.xlabel('A')
plt.ylabel('C')
plt.legend()

plt.tight_layout()
plt.show()
print("\n" + "="*50 + "\n")