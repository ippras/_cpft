import pandas as pd
import matplotlib.pyplot as plt

def create_svg_from_csv(input_file, output_file, x_start=None, x_end=None):
    # 1. Читаем файл
    # Пропускаем строки, начинающиеся с #, но используем их для поиска заголовков
    # Так как в данных X разделен запятой (6,591), pandas увидит 4 колонки вместо 3
    try:
        df = pd.read_csv(input_file, comment='#', header=None, 
                         names=['point', 'x_int', 'x_frac', 'y'])
    except Exception as e:
        print(f"Ошибка при чтении файла: {e}")
        return

    # 2. Исправляем координату X
    # Объединяем x_int и x_frac в одно число (например, 6 и 591 превращаем в 6.591)
    df['x'] = df['x_int'] + df['x_frac'] / 1000.0
    
    # 3. Фильтруем по диапазону X, если он задан
    if x_start is not None:
        df = df[df['x'] >= x_start]
    if x_end is not None:
        df = df[df['x'] <= x_end]

    if df.empty:
        print("Диапазон пуст. Проверьте значения x_start и x_end.")
        return

    # 4. Строим график
    plt.figure(figsize=(12, 6))
    plt.plot(df['x'], df['y'], color='blue', linewidth=1, label='TIC Scan')
    
    # Оформление
    plt.title(f"Chromatogram (Range: {df['x'].min():.3f} - {df['x'].max():.3f})")
    plt.xlabel("Time (Minutes)")
    plt.ylabel("Counts")
    plt.grid(True, linestyle='--', alpha=0.6)
    plt.legend()

    # 5. Сохраняем в SVG
    plt.savefig(output_file, format='svg')
    plt.close()
    print(f"Файл успешно сохранен как {output_file}")

# --- Использование ---
# Укажите имя вашего файла
file_name = 'py/60C_1ml-min_1C-min_2.CSV' 

# Пример 1: Весь график
create_svg_from_csv(file_name, 'full_plot.svg')

# Пример 2: Только область от 7.0 до 8.5 минут
create_svg_from_csv(file_name, 'zoomed_plot.svg', x_start=7.0, x_end=8.5)