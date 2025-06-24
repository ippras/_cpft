import pandas as pd
import plotly.graph_objects as go
import plotly.express as px

# Загрузка и подготовка данных
df = pd.read_csv('_temp/source.csv')
df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])
df_plot = df_cleaned.copy()

# --- Создание фигуры ---
fig = go.Figure()

# Получаем список уникальных кислот и назначаем им цвета
unique_fatty_acids = df_plot['FattyAcid'].unique()
colors = px.colors.qualitative.Plotly

# --- ИЗМЕНЕНИЕ: Словарь для разных символов больше не нужен ---
# unique_temp_steps = sorted(df_plot['TemperatureStep'].unique())
# symbols = ['circle', 'square', 'diamond', 'cross', 'x', 'circle-open', 'diamond-open', 'square-open']
# symbol_map = {step: symbols[i % len(symbols)] for i, step in enumerate(unique_temp_steps)}


# --- Добавляем данные на график в цикле ---
for i, acid in enumerate(unique_fatty_acids):
    acid_df = df_plot[df_plot['FattyAcid'] == acid]
    acid_color = colors[i % len(colors)]

    # --- Логика построения поверхности из четырехугольников ---
    try:
        # 1. Организуем данные в сетку (грид)
        grid_df = acid_df.pivot_table(
            index='OnsetTemperature', 
            columns='TemperatureStep', 
            values='EquivalentChainLength'
        )
        
        y_grid_vals = grid_df.index.values
        z_grid_vals = grid_df.columns.values
        
        # 2. Собираем все существующие вершины и их координаты
        all_x, all_y, all_z = [], [], []
        vertex_map = {}
        v_idx = 0
        for r_idx, y_val in enumerate(y_grid_vals):
            for c_idx, z_val in enumerate(z_grid_vals):
                x_val = grid_df.loc[y_val, z_val]
                if pd.notna(x_val):
                    all_x.append(x_val)
                    all_y.append(y_val)
                    all_z.append(z_val)
                    vertex_map[(r_idx, c_idx)] = v_idx
                    v_idx += 1

        # 3. Собираем грани
        faces_i, faces_j, faces_k = [], [], []
        for r_idx in range(len(y_grid_vals) - 1):
            for c_idx in range(len(z_grid_vals) - 1):
                p1_coord = (r_idx, c_idx)
                p2_coord = (r_idx, c_idx + 1)
                p3_coord = (r_idx + 1, c_idx + 1)
                p4_coord = (r_idx + 1, c_idx)
                
                if all(p in vertex_map for p in [p1_coord, p2_coord, p3_coord, p4_coord]):
                    v1 = vertex_map[p1_coord]
                    v2 = vertex_map[p2_coord]
                    v3 = vertex_map[p3_coord]
                    v4 = vertex_map[p4_coord]
                    
                    faces_i.extend([v1, v1])
                    faces_j.extend([v2, v3])
                    faces_k.extend([v3, v4])

        # 4. Добавляем поверхность на график
        if faces_i:
            fig.add_trace(go.Mesh3d(
                x=all_x, y=all_y, z=all_z,
                i=faces_i, j=faces_j, k=faces_k,
                color=acid_color, opacity=0.5,
                legendgroup=acid, name=acid, showlegend=False, hoverinfo='none'
            ))

    except Exception as e:
        print(f"Could not create surface for {acid}: {e}")


    # --- Добавляем точки (маркеры) поверх поверхностей ---
    for step in acid_df['TemperatureStep'].unique():
        step_df = acid_df[acid_df['TemperatureStep'] == step]
        
        fig.add_trace(go.Scatter3d(
            x=step_df['EquivalentChainLength'],
            y=step_df['OnsetTemperature'],
            z=step_df['TemperatureStep'],
            mode='markers',
            marker=dict(
                size=5, 
                color=acid_color, 
                # --- ИЗМЕНЕНИЕ ЗДЕСЬ: Устанавливаем один и тот же символ для всех ---
                symbol='circle', 
                line=dict(color='black', width=1)
            ),
            legendgroup=acid, name=acid,
            showlegend=bool(step == acid_df['TemperatureStep'].unique()[0]),
            hovertext=step_df['FattyAcid'],
            hovertemplate=
                '<b>%{hovertext}</b><br><br>' +
                'Equivalent Chain Length: %{x:.2f}<br>' +
                'Onset Temperature: %{y:.2f}<br>' +
                'Temperature Step: %{z:.2f}<extra></extra>'
        ))


# --- Настройка общего вида графика ---
fig.update_layout(
    title='3D-график жирных кислот с поверхностью по сетке',
    scene=dict(
        xaxis_title='Equivalent Chain Length',
        yaxis_title='Onset Temperature',
        zaxis_title='Temperature Step'
    ),
    legend=dict(
        title=dict(text='Fatty Acid (нажмите для вкл/выкл)'),
        x=0, y=1, xanchor="left", yanchor="top",
        bgcolor="rgba(255, 255, 255, 0.7)",
        bordercolor="Black", borderwidth=1
    ),
    margin=dict(l=0, r=0, b=0, t=40)
)

# Показываем график
fig.show()
