import pandas as pd
import numpy as np
import tkinter as tk
from tkinter import ttk, messagebox
import re
import io

import matplotlib
matplotlib.use('TkAgg')
import matplotlib.pyplot as plt
import seaborn as sns

# 1. Тестовые данные (замените на загрузку из файла)
raw_data = """
| Mode.OnsetTemperature | Mode.TemperatureStep | FattyAcid | RetentionFactor.Mean | RetentionFactor.StandardDeviation |
|-----------------------|----------------------|-----------|----------------------|-----------------------------------|
| 60.0                  | 1.0                  | 8:0       | 3.728                | 0.009                             |
| 60.0                  | 1.0                  | 10:0      | 6.897                | 0.012                             |
| 60.0                  | 1.0                  | 12:0      | 10.311               | 0.016                             |
| 60.0                  | 1.0                  | 14:0      | 13.569               | 0.017                             |
| 100.0                 | 5.0                  | 8:0       | 0.492                | 0.001                             |
| 100.0                 | 5.0                  | 10:0      | 0.966                | 0.002                             |
| 100.0                 | 5.0                  | 12:0      | 1.557                | 0.004                             |
| 100.0                 | 5.0                  | 14:0      | 2.182                | 0.004                             |
| 140.0                 | 10.0                 | 8:0       | 0.171                | 0.001                             |
| 140.0                 | 10.0                 | 10:0      | 0.442                | 0.001                             |
| 140.0                 | 10.0                 | 12:0      | 0.725                | 0.002                             |
| 140.0                 | 10.0                 | 14:0      | 1.028                | 0.003                             |
"""

def load_data(text):
    lines = text.strip().split('\n')
    header_line = next(l for l in lines if '|' in l)
    headers = [col.strip() for col in header_line.split('|') if col.strip()]
    
    data = []
    for line in lines:
        if '|' not in line or '---' in line or 'Mode.OnsetTemperature' in line:
            continue
        row = [col.strip() for col in line.split('|')[1:-1]]
        row = [None if val.lower() == 'null' else val for val in row]
        if len(row) == len(headers):
            data.append(row)
            
    df = pd.DataFrame(data, columns=headers)
    numeric_cols = ['Mode.OnsetTemperature', 'Mode.TemperatureStep', 'RetentionFactor.Mean']
    for col in numeric_cols:
        df[col] = pd.to_numeric(df[col], errors='coerce')
        
    # Создаем удобную колонку "Режим" (Mode)
    df['Mode_Name'] = df['Mode.OnsetTemperature'].astype(str) + "°C / " + df['Mode.TemperatureStep'].astype(str) + "°C/min"
    return df

df = load_data(raw_data)
acids = sorted(df['FattyAcid'].dropna().unique())
modes = sorted(df['Mode_Name'].dropna().unique())

def plot_radar():
    base_fa = combo_base.get()
    
    # Получаем выбранные целевые кислоты
    selected_target_indices = listbox_targets.curselection()
    target_fas = [listbox_targets.get(i) for i in selected_target_indices]
    
    # Получаем выбранные режимы
    selected_mode_indices = listbox_modes.curselection()
    selected_modes = [listbox_modes.get(i) for i in selected_mode_indices]
    
    if not base_fa or not target_fas or not selected_modes:
        messagebox.showwarning("Ошибка", "Выберите базовую кислоту, хотя бы 3 целевых и хотя бы 1 режим!")
        return
    
    if len(target_fas) < 3:
        messagebox.showwarning("Ошибка", "Для радара нужно выбрать минимум 3 целевые кислоты (оси).")
        return

    # Настройка углов для радара
    angles = np.linspace(0, 2 * np.pi, len(target_fas), endpoint=False).tolist()
    angles += angles[:1] # Замыкаем круг

    fig, ax = plt.subplots(figsize=(9, 8), subplot_kw=dict(polar=True))
    
    # Генерируем уникальные цвета
    colors = sns.color_palette("husl", len(selected_modes))
    
    # Отрисовка каждого выбранного режима
    for i, mode in enumerate(selected_modes):
        mode_data = df[df['Mode_Name'] == mode]
        
        # Ищем значение базовой кислоты в этом режиме
        base_val_series = mode_data[mode_data['FattyAcid'] == base_fa]['RetentionFactor.Mean']
        if base_val_series.empty:
            continue # Пропускаем режим, если в нем нет базовой кислоты
        base_val = base_val_series.values[0]
        
        values = []
        for t_fa in target_fas:
            t_val_series = mode_data[mode_data['FattyAcid'] == t_fa]['RetentionFactor.Mean']
            if t_val_series.empty:
                values.append(np.nan)
            else:
                values.append(t_val_series.values[0] / base_val) # Считаем отношение
                
        values += values[:1] # Замыкаем круг значений
        
        # Рисуем линию и заливаем область
        color = colors[i]
        ax.plot(angles, values, linewidth=2, label=mode, color=color)
        ax.fill(angles, values, alpha=0.15, color=color)

    # Настройка осей и подписей
    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(target_fas, fontsize=11)
    
    plt.title(f"Профиль отношений кислот к базовой ({base_fa})", size=15, pad=20)
    plt.legend(loc='upper left', bbox_to_anchor=(1.1, 1.05), title="Режимы")
    plt.tight_layout()
    plt.show()

# --- Интерфейс Tkinter ---
root = tk.Tk()
root.title("Радарная диаграмма отношений")
root.geometry("450x500")

frame = ttk.Frame(root, padding=15)
frame.pack(fill=tk.BOTH, expand=True)

# 1. Выбор базовой кислоты
ttk.Label(frame, text="1. Базовая кислота (Знаменатель):", font=("Arial", 10, "bold")).pack(anchor=tk.W)
combo_base = ttk.Combobox(frame, values=acids, state="readonly")
combo_base.pack(fill=tk.X, pady=(5, 15))
if acids: combo_base.set(acids[0])

# 2. Выбор целевых кислот (Оси радара)
ttk.Label(frame, text="2. Целевые кислоты (Оси радара):", font=("Arial", 10, "bold")).pack(anchor=tk.W)
ttk.Label(frame, text="(Зажмите Ctrl для выбора нескольких)").pack(anchor=tk.W)
# ИСПРАВЛЕНО: добавлено exportselection=False
listbox_targets = tk.Listbox(frame, selectmode=tk.MULTIPLE, height=6, exportselection=False)
listbox_targets.pack(fill=tk.X, pady=(5, 15))
for acid in acids:
    listbox_targets.insert(tk.END, acid)
for i in range(min(4, len(acids))):
    listbox_targets.select_set(i)

# 3. Выбор режимов
ttk.Label(frame, text="3. Режимы для сравнения:", font=("Arial", 10, "bold")).pack(anchor=tk.W)
ttk.Label(frame, text="(Зажмите Ctrl для выбора нескольких)").pack(anchor=tk.W)
# ИСПРАВЛЕНО: добавлено exportselection=False
listbox_modes = tk.Listbox(frame, selectmode=tk.MULTIPLE, height=6, exportselection=False)
listbox_modes.pack(fill=tk.X, pady=(5, 15))
for mode in modes:
    listbox_modes.insert(tk.END, mode)
for i in range(min(2, len(modes))):
    listbox_modes.select_set(i)

# Кнопка
btn_plot = ttk.Button(frame, text="Построить Радар", command=plot_radar)
btn_plot.pack(fill=tk.X, ipady=8)

root.mainloop()