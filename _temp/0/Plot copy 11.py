import pandas as pd
import plotly.graph_objects as go
import plotly.express as px

# Загрузка и подготовка данных
df = pd.read_csv('_temp/source.csv')
df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])
df_filtered = df_cleaned[df_cleaned['EquivalentChainLength'] % 1 != 0].copy()
df_plot = df_filtered.copy()

# --- Создание фигуры ---
fig = go.Figure()

# Получаем список уникальных кислот и назначаем им цвета
unique_fatty_acids = df_plot['FattyAcid'].unique()
colors = px.colors.qualitative.Plotly

# --- НОВЫЙ ШАГ: Список для хранения индексов трейсов поверхностей ---
surface_trace_indices = []

# --- Добавляем данные на график в цикле ---
for i, acid in enumerate(unique_fatty_acids):
    acid_df = df_plot[df_plot['FattyAcid'] == acid]
    acid_color = colors[i % len(colors)]

    # --- Логика построения поверхности из четырехугольников ---
    try:
        grid_df = acid_df.pivot_table(
            index='OnsetTemperature', columns='TemperatureStep', values='EquivalentChainLength'
        )
        y_grid_vals, z_grid_vals = grid_df.index.values, grid_df.columns.values
        all_x, all_y, all_z, vertex_map = [], [], [], {}
        v_idx = 0
        for r_idx, y_val in enumerate(y_grid_vals):
            for c_idx, z_val in enumerate(z_grid_vals):
                x_val = grid_df.loc[y_val, z_val]
                if pd.notna(x_val):
                    all_x.append(x_val); all_y.append(y_val); all_z.append(z_val)
                    vertex_map[(r_idx, c_idx)] = v_idx
                    v_idx += 1

        faces_i, faces_j, faces_k = [], [], []
        for r_idx in range(len(y_grid_vals) - 1):
            for c_idx in range(len(z_grid_vals) - 1):
                p1, p2, p3, p4 = (r_idx, c_idx), (r_idx, c_idx + 1), (r_idx + 1, c_idx + 1), (r_idx + 1, c_idx)
                if all(p in vertex_map for p in [p1, p2, p3, p4]):
                    v1, v2, v3, v4 = vertex_map[p1], vertex_map[p2], vertex_map[p3], vertex_map[p4]
                    faces_i.extend([v1, v1]); faces_j.extend([v2, v3]); faces_k.extend([v3, v4])

        if faces_i:
            fig.add_trace(go.Mesh3d(
                x=all_x, y=all_y, z=all_z,
                i=faces_i, j=faces_j, k=faces_k,
                color=acid_color, opacity=0.5,
                legendgroup=acid, name=acid, showlegend=False, hoverinfo='none'
            ))
            # --- НОВЫЙ ШАГ: Запоминаем индекс только что добавленной поверхности ---
            surface_trace_indices.append(len(fig.data) - 1)

    except Exception as e:
        print(f"Could not create surface for {acid}: {e}")

    # --- Добавляем точки (маркеры) ---
    for step in acid_df['TemperatureStep'].unique():
        step_df = acid_df[acid_df['TemperatureStep'] == step]
        fig.add_trace(go.Scatter3d(
            x=step_df['EquivalentChainLength'], y=step_df['OnsetTemperature'], z=step_df['TemperatureStep'],
            mode='markers',
            marker=dict(size=5, color=acid_color, symbol='circle', line=dict(color='black', width=1)),
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
        title=dict(text='Fatty Acid (нажмите)'),
        x=0, y=1, xanchor="left", yanchor="top",
        bgcolor="rgba(255, 255, 255, 0.7)", bordercolor="Black", borderwidth=1
    ),
    margin=dict(l=0, r=0, b=0, t=40),
    
    # --- НОВЫЙ БЛОК: Добавляем кнопки для управления поверхностями ---
    updatemenus=[
        dict(
            type="buttons",
            direction="right",
            active=0, # Кнопка "Показать" активна по умолчанию
            buttons=[
                dict(
                    label="Показать поверхности",
                    method="restyle",
                    # Применить visible: True к трейсам с индексами из нашего списка
                    args=[{"visible": True}, surface_trace_indices]
                ),
                dict(
                    label="Скрыть поверхности",
                    method="restyle",
                    # Применить visible: False к тем же трейсам
                    args=[{"visible": False}, surface_trace_indices]
                ),
            ],
            pad={"r": 10, "t": 10},
            showactive=True,
            x=0.01, # Располагаем слева
            xanchor="left",
            y=1.1,  # Располагаем над графиком
            yanchor="top"
        )
    ]
)

# Показываем график
fig.show()