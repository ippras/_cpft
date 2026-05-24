import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from matplotlib.colors import LinearSegmentedColormap
import tkinter as tk
from tkinter import ttk, messagebox

# ==========================================
# 1. Исходные данные
# ==========================================
data = [
    [60.0, 1.0, "18:1Δ9c", 18.394], [60.0, 2.0, "18:1Δ9c", 18.44], [60.0, 3.0, "18:1Δ9c", 18.469], [60.0, 4.0, "18:1Δ9c", 18.489], [60.0, 5.0, "18:1Δ9c", 18.505], [60.0, 6.0, "18:1Δ9c", 18.517], [60.0, 7.0, "18:1Δ9c", 18.528], [60.0, 8.0, "18:1Δ9c", 18.536], [60.0, 9.0, "18:1Δ9c", 18.544], [60.0, 10.0, "18:1Δ9c", 18.556],
    [70.0, 1.0, "18:1Δ9c", 18.394], [70.0, 2.0, "18:1Δ9c", 18.441], [70.0, 3.0, "18:1Δ9c", 18.47], [70.0, 4.0, "18:1Δ9c", 18.489], [70.0, 5.0, "18:1Δ9c", 18.506], [70.0, 6.0, "18:1Δ9c", 18.52], [70.0, 7.0, "18:1Δ9c", 18.531], [70.0, 8.0, "18:1Δ9c", 18.537], [70.0, 9.0, "18:1Δ9c", 18.546], [70.0, 10.0, "18:1Δ9c", 18.554],
    [80.0, 1.0, "18:1Δ9c", 18.395], [80.0, 2.0, "18:1Δ9c", 18.442], [80.0, 3.0, "18:1Δ9c", 18.47], [80.0, 4.0, "18:1Δ9c", 18.491], [80.0, 5.0, "18:1Δ9c", 18.507], [80.0, 6.0, "18:1Δ9c", 18.519], [80.0, 7.0, "18:1Δ9c", 18.531], [80.0, 8.0, "18:1Δ9c", 18.539], [80.0, 9.0, "18:1Δ9c", 18.545], [80.0, 10.0, "18:1Δ9c", 18.553],
    [90.0, 1.0, "18:1Δ9c", 18.398], [90.0, 2.0, "18:1Δ9c", 18.443], [90.0, 3.0, "18:1Δ9c", 18.471], [90.0, 4.0, "18:1Δ9c", 18.492], [90.0, 5.0, "18:1Δ9c", 18.506], [90.0, 6.0, "18:1Δ9c", 18.52], [90.0, 7.0, "18:1Δ9c", 18.532], [90.0, 8.0, "18:1Δ9c", 18.538], [90.0, 9.0, "18:1Δ9c", 18.545], [90.0, 10.0, "18:1Δ9c", 18.553],
    [100.0, 1.0, "18:1Δ9c", 18.401], [100.0, 2.0, "18:1Δ9c", 18.446], [100.0, 3.0, "18:1Δ9c", 18.474], [100.0, 4.0, "18:1Δ9c", 18.494], [100.0, 5.0, "18:1Δ9c", 18.509], [100.0, 6.0, "18:1Δ9c", 18.521], [100.0, 7.0, "18:1Δ9c", 18.532], [100.0, 8.0, "18:1Δ9c", 18.542], [100.0, 9.0, "18:1Δ9c", 18.548], [100.0, 10.0, "18:1Δ9c", 18.557],
    [110.0, 1.0, "18:1Δ9c", 18.407], [110.0, 2.0, "18:1Δ9c", 18.451], [110.0, 3.0, "18:1Δ9c", 18.477], [110.0, 4.0, "18:1Δ9c", 18.498], [110.0, 5.0, "18:1Δ9c", 18.513], [110.0, 6.0, "18:1Δ9c", 18.524], [110.0, 7.0, "18:1Δ9c", 18.538], [110.0, 8.0, "18:1Δ9c", 18.547], [110.0, 9.0, "18:1Δ9c", 18.554], [110.0, 10.0, "18:1Δ9c", 18.556],
    [120.0, 1.0, "18:1Δ9c", 18.414], [120.0, 2.0, "18:1Δ9c", 18.456], [120.0, 3.0, "18:1Δ9c", 18.482], [120.0, 4.0, "18:1Δ9c", 18.504], [120.0, 5.0, "18:1Δ9c", 18.518], [120.0, 6.0, "18:1Δ9c", 18.531], [120.0, 7.0, "18:1Δ9c", 18.539], [120.0, 8.0, "18:1Δ9c", 18.552], [120.0, 9.0, "18:1Δ9c", 18.557], [120.0, 10.0, "18:1Δ9c", 18.566],
    [130.0, 1.0, "18:1Δ9c", 18.423], [130.0, 2.0, "18:1Δ9c", 18.465], [130.0, 3.0, "18:1Δ9c", 18.491], [130.0, 4.0, "18:1Δ9c", 18.508], [130.0, 5.0, "18:1Δ9c", 18.523], [130.0, 6.0, "18:1Δ9c", 18.537], [130.0, 7.0, "18:1Δ9c", 18.546], [130.0, 8.0, "18:1Δ9c", 18.557], [130.0, 9.0, "18:1Δ9c", 18.564], [130.0, 10.0, "18:1Δ9c", 18.57],
    [140.0, 1.0, "18:1Δ9c", 18.436], [140.0, 2.0, "18:1Δ9c", 18.475], [140.0, 3.0, "18:1Δ9c", 18.501], [140.0, 4.0, "18:1Δ9c", 18.519], [140.0, 5.0, "18:1Δ9c", 18.532], [140.0, 6.0, "18:1Δ9c", 18.543], [140.0, 7.0, "18:1Δ9c", 18.554], [140.0, 8.0, "18:1Δ9c", 18.563], [140.0, 9.0, "18:1Δ9c", 18.572], [140.0, 10.0, "18:1Δ9c", 18.579],
    [150.0, 1.0, "18:1Δ9c", 18.453], [150.0, 2.0, "18:1Δ9c", 18.489], [150.0, 3.0, "18:1Δ9c", 18.515], [150.0, 4.0, "18:1Δ9c", 18.531], [150.0, 5.0, "18:1Δ9c", 18.545], [150.0, 6.0, "18:1Δ9c", 18.559], [150.0, 7.0, "18:1Δ9c", 18.569], [150.0, 8.0, "18:1Δ9c", 18.577], [150.0, 9.0, "18:1Δ9c", 18.583], [150.0, 10.0, "18:1Δ9c", 18.59]
]

df = pd.DataFrame(data, columns=["Init_Temp", "Temp_Step", "Compound", "ECL"])
unique_temps = sorted(df["Init_Temp"].unique())
unique_steps = sorted(df["Temp_Step"].unique())

# ==========================================
# 2. Окно со статистикой регрессии
# ==========================================
def show_stats_window(stats_data):
    stats_win = tk.Toplevel(root)
    stats_win.title("Статистика квадратичной регрессии")
    stats_win.geometry("850x300")
    
    # Создаем таблицу
    columns = ("curve", "equation", "r2", "rmse")
    tree = ttk.Treeview(stats_win, columns=columns, show="headings")
    
    tree.heading("curve", text="Кривая")
    tree.heading("equation", text="Уравнение (y = ax² + bx + c)")
    tree.heading("r2", text="R² (Точность)")
    tree.heading("rmse", text="RMSE (Ошибка)")
    
    tree.column("curve", width=100, anchor="center")
    tree.column("equation", width=350, anchor="center")
    tree.column("r2", width=150, anchor="center")
    tree.column("rmse", width=150, anchor="center")
    
    for stat in stats_data:
        tree.insert("", tk.END, values=(
            stat['curve'],
            stat['equation'],
            f"{stat['r2']:.5f}",
            f"{stat['rmse']:.5f}"
        ))
        
    tree.pack(fill="both", expand=True, padx=10, pady=10)

# ==========================================
# 3. Функция построения графика
# ==========================================
def plot_graph():
    selected_temps = [float(listbox_temps.get(i)) for i in listbox_temps.curselection()]
    selected_steps = [float(listbox_steps.get(i)) for i in listbox_steps.curselection()]
    
    if not selected_temps or not selected_steps:
        messagebox.showwarning("Ошибка", "Выберите хотя бы одну температуру и один шаг!")
        return

    filtered_df = df[(df["Init_Temp"].isin(selected_temps)) & (df["Temp_Step"].isin(selected_steps))]
    
    plot_type = var_plot_type.get()
    x_axis = var_x_axis.get()
    do_regression = var_regression.get()
    
    sns.set_theme(style="whitegrid")
    plt.figure(figsize=(10, 6))
    
    if x_axis == "Temp_Step":
        x_col, hue_col = "Temp_Step", "Init_Temp"
        x_label, hue_label = "Шаг температуры", "Нач. температура (°C)"
    else:
        x_col, hue_col = "Init_Temp", "Temp_Step"
        x_label, hue_label = "Начальная температура (°C)", "Шаг температуры"

    # --- ЛИНЕЙНЫЙ ГРАФИК С КВАДРАТИЧНОЙ РЕГРЕССИЕЙ ---
    if plot_type == "Line":
        stats_list = []
        
        for val in sorted(filtered_df[hue_col].unique()):
            subset = filtered_df[filtered_df[hue_col] == val]
            x_data = subset[x_col].values
            y_data = subset['ECL'].values
            
            # Для квадратичной регрессии нужно минимум 3 точки
            if do_regression and len(x_data) >= 3:
                # Расчет квадратичной регрессии (полином 2 степени)
                coeffs = np.polyfit(x_data, y_data, 2)
                a, b, c = coeffs
                
                # Расчет предсказанных значений для статистики
                y_pred = np.polyval(coeffs, x_data)
                
                # Расчет R^2
                ss_res = np.sum((y_data - y_pred) ** 2)
                ss_tot = np.sum((y_data - np.mean(y_data)) ** 2)
                r_squared = 1 - (ss_res / ss_tot)
                
                # Расчет RMSE (Root Mean Square Error)
                rmse = np.sqrt(ss_res / len(y_data))
                
                # Рисуем исходные точки
                p = plt.plot(x_data, y_data, marker='o', linestyle='', label=f'{val} (данные)')
                color = p[0].get_color()
                
                # Генерируем плавную кривую для отрисовки
                x_smooth = np.linspace(x_data.min(), x_data.max(), 100)
                y_smooth = np.polyval(coeffs, x_smooth)
                
                # Рисуем кривую регрессии
                plt.plot(x_smooth, y_smooth, color=color, linestyle='-', label=f'{val} (R²={r_squared:.3f})')
                
                # Форматируем уравнение (используем .4g для красивого отображения очень маленьких чисел)
                equation_str = f"y = {a:.4g}x² + {b:.4g}x + {c:.4g}"
                
                # Сохраняем статистику
                stats_list.append({
                    'curve': val,
                    'equation': equation_str,
                    'r2': r_squared,
                    'rmse': rmse
                })
            else:
                # Обычный график без регрессии (или если точек меньше 3)
                plt.plot(x_data, y_data, marker='o', label=f'{val}')
                
        plt.title('Зависимость ECL (18:1Δ9c)', fontsize=14)
        plt.xlabel(x_label, fontsize=12)
        plt.ylabel('ECL', fontsize=12)
        plt.legend(title=hue_label, bbox_to_anchor=(1.05, 1), loc='upper left')
        plt.tight_layout()
        
        # Показываем окно со статистикой, если регрессия включена
        if do_regression and stats_list:
            show_stats_window(stats_list)

    # --- ТЕПЛОВАЯ КАРТА ---
    elif plot_type == "Heatmap":
        pivot_df = filtered_df.pivot(index=hue_col, columns=x_col, values="ECL")
        custom_cmap = LinearSegmentedColormap.from_list("blue_to_red", ["blue", "red"])
        sns.heatmap(pivot_df, annot=True, fmt=".3f", cmap=custom_cmap, cbar_kws={'label': 'ECL'})
        
        plt.title('Тепловая карта значений ECL (18:1Δ9c)', fontsize=14)
        plt.xlabel(x_label, fontsize=12)
        plt.ylabel(hue_label, fontsize=12)
        plt.gca().invert_yaxis()
        plt.tight_layout()

    plt.show()

# ==========================================
# 4. Создание графического интерфейса (GUI)
# ==========================================
root = tk.Tk()
root.title("Настройки графика ECL")
root.geometry("450x550")
root.resizable(False, False)

# --- Блок выбора типа графика ---
frame_type = ttk.LabelFrame(root, text="Тип графика", padding=10)
frame_type.pack(fill="x", padx=10, pady=5)

var_plot_type = tk.StringVar(value="Line")
ttk.Radiobutton(frame_type, text="Линейный график", variable=var_plot_type, value="Line").pack(side="left", padx=10)
ttk.Radiobutton(frame_type, text="Тепловая карта", variable=var_plot_type, value="Heatmap").pack(side="left", padx=10)

# --- Блок выбора оси X ---
frame_axis = ttk.LabelFrame(root, text="Что отображать на оси X?", padding=10)
frame_axis.pack(fill="x", padx=10, pady=5)

var_x_axis = tk.StringVar(value="Temp_Step")
ttk.Radiobutton(frame_axis, text="Шаг температуры", variable=var_x_axis, value="Temp_Step").pack(side="left", padx=10)
ttk.Radiobutton(frame_axis, text="Начальная температура", variable=var_x_axis, value="Init_Temp").pack(side="left", padx=10)

# --- Блок регрессии ---
frame_reg = ttk.Frame(root)
frame_reg.pack(fill="x", padx=10, pady=5)
var_regression = tk.BooleanVar(value=False)
ttk.Checkbutton(frame_reg, text="Добавить квадратичную регрессию (только для линий)", variable=var_regression).pack(anchor="w")

# --- Блок фильтрации данных ---
frame_data = ttk.LabelFrame(root, text="Фильтрация данных (Ctrl+Клик для выбора нескольких)", padding=10)
frame_data.pack(fill="both", expand=True, padx=10, pady=5)

frame_lists = ttk.Frame(frame_data)
frame_lists.pack(fill="both", expand=True)

frame_temps = ttk.Frame(frame_lists)
frame_temps.pack(side="left", fill="both", expand=True, padx=5)
ttk.Label(frame_temps, text="Нач. температура:").pack()
listbox_temps = tk.Listbox(frame_temps, selectmode="extended", exportselection=False, height=10)
listbox_temps.pack(fill="both", expand=True)
for t in unique_temps:
    listbox_temps.insert(tk.END, str(t))
listbox_temps.select_set(0, tk.END)

frame_steps = ttk.Frame(frame_lists)
frame_steps.pack(side="right", fill="both", expand=True, padx=5)
ttk.Label(frame_steps, text="Шаг температуры:").pack()
listbox_steps = tk.Listbox(frame_steps, selectmode="extended", exportselection=False, height=10)
listbox_steps.pack(fill="both", expand=True)
for s in unique_steps:
    listbox_steps.insert(tk.END, str(s))
listbox_steps.select_set(0, tk.END)

# --- Кнопка построения ---
btn_plot = ttk.Button(root, text="Построить график", command=plot_graph)
btn_plot.pack(pady=15, ipadx=20, ipady=5)

root.mainloop()