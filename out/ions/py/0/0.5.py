import pandas as pd
import matplotlib.pyplot as plt
import tkinter as tk
from tkinter import filedialog
import os

def plot_eic_files():
    # Создаем невидимое окно для вызова диалога выбора файлов
    root = tk.Tk()
    root.withdraw()
    
    # Выбираем несколько файлов
    file_paths = filedialog.askopenfilenames(
        title="Выберите CSV файлы с данными EIC",
        filetypes=[("CSV files", "*.csv"), ("All files", "*.*")]
    )
    
    if not file_paths:
        print("Файлы не выбраны.")
        return

    plt.figure(figsize=(12, 6))

    for path in file_paths:
        # Пытаемся прочитать название иона из первой строки (комментария)
        with open(path, 'r') as f:
            first_line = f.readline()
            # Ищем что-то вроде EIC(108,1)
            if "EIC" in first_line:
                label = first_line.split("Scan")[0].replace('#"', '').strip()
            else:
                label = os.path.basename(path)

        # Загружаем данные
        # Учитываем, что в ваших файлах разделитель запятая, а в числах тоже могут быть запятые
        # Судя по вашему примеру: Point, X(Minutes), Y(Counts)
        # Если время 5,841 записано через запятую, pandas может воспринять это как два столбца.
        try:
            # Читаем данные, пропуская строки с решеткой #
            df = pd.read_csv(path, comment='#', header=None)
            
            # Если столбцов 4 (из-за запятой в десятичном числе), объединяем их
            if df.shape[1] == 4:
                # Склеиваем 5 и 841 в 5.841
                time = df[1] + df[2] / 1000
                intensity = df[3]
            else:
                time = df.iloc[:, 1]
                intensity = df.iloc[:, 2]

            plt.plot(time, intensity, label=label)
            
        except Exception as e:
            print(f"Ошибка при чтении файла {path}: {e}")

    plt.xlabel('Время (мин)')
    plt.ylabel('Интенсивность (Counts)')
    plt.title('Наложение хроматограмм (EIC)')
    plt.legend()
    plt.grid(True, alpha=0.3)
    plt.show()

if __name__ == "__main__":
    plot_eic_files()