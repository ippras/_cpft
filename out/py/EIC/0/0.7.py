import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider, Button, TextBox
import tkinter as tk
from tkinter import filedialog
import os

# Глобальные переменные для хранения состояния
lines = []  # Список объектов линий на графике
all_data_bounds = [float('inf'), float('-inf')] # [min_t, max_t]

def load_data(file_path):
    """Чтение данных с разделителем ';' и десятичной запятой"""
    try:
        # Добавили decimal=',', так как в ваших данных время указано как 6,591
        df = pd.read_csv(file_path, sep=';', decimal=',', comment='#', header=None)
        
        # Проверка структуры (как в вашем примере: Point;X;Y)
        if df.shape[1] >= 3:
            time = pd.to_numeric(df.iloc[:, 1], errors='coerce')
            intensity = pd.to_numeric(df.iloc[:, 2], errors='coerce')
            mask = time.notna() & intensity.notna()
            return time[mask], intensity[mask]
        return None, None
    except Exception as e:
        print(f"Ошибка в файле {file_path}: {e}")
        return None, None

def start_analysis():
    fig, ax = plt.subplots(figsize=(12, 8))
    plt.subplots_adjust(bottom=0.35) # Увеличили место под кнопки

    ax.set_xlabel('Время (мин)')
    ax.set_ylabel('Интенсивность (Counts)')
    ax.set_title('EIC Chromatograms')
    ax.grid(True, alpha=0.3)

    # --- Виджеты управления ---
    ax_slider = plt.axes([0.2, 0.20, 0.6, 0.03])
    # Инициализируем слайдер заглушкой
    slider = RangeSlider(ax_slider, 'Диапазон', 0, 1, valinit=(0, 1))

    ax_box_min = plt.axes([0.2, 0.13, 0.1, 0.04])
    ax_box_max = plt.axes([0.7, 0.13, 0.1, 0.04])
    text_min = TextBox(ax_box_min, 'Min: ', initial="0")
    text_max = TextBox(ax_box_max, 'Max: ', initial="1")

    def update_slider_range(new_min, new_max):
        """Обновляет границы слайдера при добавлении новых файлов"""
        all_data_bounds[0] = min(all_data_bounds[0], new_min)
        all_data_bounds[1] = max(all_data_bounds[1], new_max)
        
        slider.valmin = all_data_bounds[0]
        slider.valmax = all_data_bounds[1]
        ax_slider.set_xlim(slider.valmin, slider.valmax)
        
        # Если это первый файл, ставим ползунки по краям
        if len(lines) <= 1:
            slider.set_val((slider.valmin, slider.valmax))

    def add_files(event):
        root = tk.Tk()
        root.withdraw()
        file_paths = filedialog.askopenfilenames(
            title="Добавить файлы", 
            filetypes=[("Data files", "*.csv *.txt"), ("All files", "*.*")]
        )
        root.destroy()

        if not file_paths:
            return

        for path in file_paths:
            # Читаем заголовок для легенды
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                header = f.readline()
                label = header.split("Scan")[0].replace('#"', '').strip() if "EIC" in header else os.path.basename(path)
            
            t, y = load_data(path)
            if t is not None and not t.empty:
                line, = ax.plot(t, y, label=label, lw=1.5)
                lines.append(line)
                update_slider_range(t.min(), t.max())
        
        ax.legend(loc='upper right')
        fig.canvas.draw_idle()

    def clear_plots(event):
        """Удаляет все графики"""
        for line in lines:
            line.remove()
        lines.clear()
        all_data_bounds[0], all_data_bounds[1] = float('inf'), float('-inf')
        ax.legend_ = None
        ax.set_xlim(0, 1)
        ax.set_ylim(0, 1)
        fig.canvas.draw_idle()

    # --- Кнопки интерфейса ---
    
    # Кнопка Добавить
    ax_add = plt.axes([0.2, 0.02, 0.15, 0.05])
    btn_add = Button(ax_add, '➕ Добавить файлы', color='lightgreen', hovercolor='0.9')
    btn_add.on_clicked(add_files)

    # Кнопка Очистить
    ax_clear = plt.axes([0.37, 0.02, 0.15, 0.05])
    btn_clear = Button(ax_clear, '🗑️ Очистить всё', color='salmon', hovercolor='0.9')
    btn_clear.on_clicked(clear_plots)

    # Кнопка Сохранить
    ax_save = plt.axes([0.65, 0.02, 0.15, 0.05])
    btn_save = Button(ax_save, '💾 Save SVG', color='lightblue', hovercolor='0.9')

    # Логика слайдера и ввода
    def on_slider_change(val):
        text_min.set_val(f"{val[0]:.2f}")
        text_max.set_val(f"{val[1]:.2f}")
        ax.set_xlim(val[0], val[1])
        fig.canvas.draw_idle()

    def on_text_submit(text):
        try:
            v_min = float(text_min.text.replace(',', '.'))
            v_max = float(text_max.text.replace(',', '.'))
            slider.set_val((v_min, v_max))
        except ValueError: pass

    slider.on_changed(on_slider_change)
    text_min.on_submit(on_text_submit)
    text_max.on_submit(on_text_submit)

    def save_svg(event):
        # Скрываем UI перед сохранением
        for ui in [ax_slider, ax_box_min, ax_box_max, ax_add, ax_clear, ax_save]:
            ui.set_visible(False)
        
        root = tk.Tk()
        root.withdraw()
        save_path = filedialog.asksaveasfilename(defaultextension=".svg", filetypes=[("SVG", "*.svg")])
        root.destroy()
        
        if save_path:
            fig.savefig(save_path, format='svg', bbox_inches='tight')
        
        for ui in [ax_slider, ax_box_min, ax_box_max, ax_add, ax_clear, ax_save]:
            ui.set_visible(True)
        fig.canvas.draw()

    btn_save.on_clicked(save_svg)

    # Сразу вызываем окно выбора при старте
    add_files(None)
    
    plt.show()

if __name__ == "__main__":
    start_analysis()