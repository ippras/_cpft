import pandas as pd
import argparse
import io
import re
import sys

def load_md_table(filepath):
    """Читает первую найденную Markdown таблицу из файла и возвращает DataFrame."""
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    table_lines = []
    in_table = False
    for line in lines:
        if '|' in line.strip():
            in_table = True
            table_lines.append(line.strip())
        elif in_table:
            break  # Читаем только первую таблицу в файле

    if not table_lines:
        raise ValueError("Markdown таблица не найдена в файле.")

    # Удаляем строку-разделитель Markdown (например, |---|---|)
    # Обычно она идет второй строкой
    if len(table_lines) > 1 and re.match(r'^\|?[\s\-:]+\|?$', table_lines[1].replace('|', '')):
        table_lines.pop(1)

    # Читаем строки как CSV с разделителем '|'
    table_str = '\n'.join(table_lines)
    df = pd.read_csv(io.StringIO(table_str), sep='|', engine='python')

    # Удаляем пустые колонки по краям (возникают из-за начальных и конечных '|')
    df = df.dropna(how='all', axis=1)
    df.columns = df.columns.str.strip()
    df = df.loc[:, ~df.columns.str.contains('^Unnamed')]

    # Очищаем пробелы в строковых значениях
    for col in df.columns:
        if df[col].dtype == 'object':
            df[col] = df[col].str.strip()

    return df

def main():
    parser = argparse.ArgumentParser(description="Скрипт для Pivot или Transpose Markdown таблиц.")
    parser.add_argument("input", help="Путь к входному .md файлу")
    parser.add_argument("-o", "--output", help="Путь к выходному .md файлу (если не указан, выведет в консоль)")
    
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--transpose", metavar="COLUMN", 
                       help="Имя столбца, значения которого станут строкой заголовков (Транспонирование)")
    group.add_argument("--pivot", nargs=3, metavar=('INDEX', 'COLUMNS', 'VALUES'),
                       help="Сделать сводную таблицу. Укажите 3 столбца: [Индекс] [Столбец_в_строку] [Значения]")

    args = parser.parse_args()

    try:
        df = load_md_table(args.input)
    except Exception as e:
        print(f"Ошибка при чтении файла: {e}")
        sys.exit(1)

    if args.transpose:
        col_name = args.transpose
        if col_name not in df.columns:
            print(f"Ошибка: Столбец '{col_name}' не найден. Доступные столбцы: {list(df.columns)}")
            sys.exit(1)
            
        # Делаем указанный столбец индексом и транспонируем
        df_result = df.set_index(col_name).T
        df_result.index.name = 'Property'
        df_result.reset_index(inplace=True)
        df_result.columns.name = None

    elif args.pivot:
        idx_col, pivot_col, val_col = args.pivot
        for col in [idx_col, pivot_col, val_col]:
            if col not in df.columns:
                print(f"Ошибка: Столбец '{col}' не найден. Доступные столбцы: {list(df.columns)}")
                sys.exit(1)
                
        # Классический Pivot
        df_result = df.pivot(index=idx_col, columns=pivot_col, values=val_col)
        df_result.reset_index(inplace=True)
        df_result.columns.name = None

    # Конвертируем обратно в Markdown
    md_out = df_result.to_markdown(index=False)

    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(md_out + '\n')
        print(f"Успешно сохранено в {args.output}")
    else:
        print(md_out)

if __name__ == "__main__":
    main()