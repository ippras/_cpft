import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.widgets import RangeSlider, Button
from tkinter import filedialog # Для выбора места сохранения

def plot_chromatogram(csv_file):
    # 1. Загрузка данных
    try:
        # Читаем CSV. Из-за запятой в X (6,591) pandas видит 4 колонки
        df = pd.read_csv(csv_file, comment='#', header=None, 
                         names=['point', 'x_int', 'x_frac', 'y'])
        df['x'] = df['x_int'] + df['x_frac'] / 1000.0
    except Exception as e:
        print(f"Ошибка: {e}")
        return

    # 2. Создание основного окна с графиком
    fig, ax = plt.subplots(figsize=(12, 7))
    plt.subplots_adjust(bottom=0.25) # Резервируем место под слайдер и кнопку

    line, = ax.plot(df['x'], df['y'], color='#1f77b4', lw=1.5)
    ax.set_xlabel('Time (Minutes)')
    ax.set_ylabel('Counts')
    ax.set_title('TIC Scan Chromatogram')
    ax.grid(True, linestyle='--', alpha=0.5)

    # 3. Добавление ползунка диапазона (RangeSlider)
    ax_slider = plt.axes([0.15, 0.1, 0.7, 0.03])
    slider = RangeSlider(
        ax_slider, "Range X", 
        df['x'].min(), df['x'].max(), 
        valinit=(df['x'].min(), df['x'].max())
    )

    # Функция обновления масштаба
    def update(val):
        ax.set_xlim(val[0], val[1])
        # Авто-масштаб по высоте (Y) для выбранного участка
        visible_y = df[(df['x'] >= val[0]) & (df['x'] <= val[1])]['y']
        if not visible_y.empty:
            ax.set_ylim(visible_y.min() * 0.9, visible_y.max() * 1.1)
        fig.canvas.draw_idle()

    slider.on_changed(update)

    # 4. Функция "Чистого" сохранения
    def save_svg(event):
        # Запрашиваем имя файла у пользователя
        file_path = filedialog.asksaveasfilename(
            defaultextension=".svg",
            filetypes=[("SVG files", "*.svg")],
            title="Сохранить график как..."
        )
        
        if file_path:
            # Создаем временную фигуру БЕЗ слайдеров
            export_fig, export_ax = plt.subplots(figsize=(10, 6))
            export_ax.plot(df['x'], df['y'], color='#1f77b4', lw=1.5)
            
            # Копируем границы с интерактивного окна
            export_ax.set_xlim(ax.get_xlim())
            export_ax.set_ylim(ax.get_ylim())
            
            # Копируем оформление
            export_ax.set_xlabel('Time (Minutes)')
            export_ax.set_ylabel('Counts')
            export_ax.set_title('TIC Scan Chromatogram')
            export_ax.grid(True, linestyle='--', alpha=0.5)
            
            # Сохраняем
            export_fig.savefig(file_path, format='svg')
            plt.close(export_fig)
            print(f"График сохранен в: {file_path}")

    # 5. Добавление кнопки сохранения
    ax_button = plt.axes([0.8, 0.02, 0.1, 0.04])
    btn_save = Button(ax_button, 'Save SVG', color='#e1e1e1', hovercolor='#99ff99')
    btn_save.on_clicked(save_svg)

    print("Инструкция:")
    print("- Тяните края ползунка внизу, чтобы выбрать область.")
    print("- Нажмите кнопку 'Save SVG' для экспорта только графика.")

    plt.show()

# Укажите путь к вашему файлу
plot_chromatogram('ions/{80;4;Ion=79}[2].CSV')