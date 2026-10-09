command-adminify-desc = 临时授予玩家受限的管理员权限，或移除当前权限（若未授予）
command-alias-desc = 更改您的别名
command-area_add-desc = 新增一个建造区域
command-area_list-desc = 列出所有建造区域
command-area_remove-desc = 移除指定建造区域
command-aura-desc = 创建一个光环
command-body-desc = 将您的角色变成不同种族
command-set_body_type-desc = 设置性别：女性或男性。
command-set_body_type-not_found =
    这不是有效的体型。
    请尝试以下选项之一：
    { $options }
command-set_body_type-no_body = 无法设置体型，因为目标没有身体。
command-set_body_type-not_character = 仅能对在线玩家角色永久设置体型。
command-buff-desc = 给玩家施加增益效果
command-build-desc = 开关建筑模式
command-battlemode-desc =
    设置你的战斗模式为：
    + pvp（玩家对战）
    + pve（玩家对战环境）
    如不带参数调用，将显示当前战斗模式。
command-battlemode_force-desc = 直接更改战斗模式，无需验证
command-campfire-desc = 生成一个篝火
command-help-template = { $usage }{ $description }
command-help-list =
    { $client-commands }
    { $server-commands }

    此外，您可以使用以下快捷键:
    { $additional-shortcuts }
command-airship-desc = 生成一艘空中飞船
command-ban-desc = 根据给定的用户名，对玩家进行禁用操作，持续时间由参数指定(如果提供)。传递true以覆盖并修改现有禁令。
command-ban-ip-desc = 封禁拥有指定用户名的玩家，期限为指定时长(若已提供)。与常规封禁不同，此操作还会额外封禁与该用户关联的IP地址。传递true可覆盖选项，则可将现有封禁状态进行更改。
command-clear_persisted_terrain-desc = 清除附近已存在的地形
command-create_location-desc = 在当前位置创建一个定位
command-death_effect-dest = 为目标实体添加一个死亡时效果
command-debug_column-desc = 打印有关某列的一些调试信息
command-debug_ways-desc = 打印有关列的存储方式的调试信息
command-delete_location-desc = 删除定位
command-destroy_tethers-desc = 摧毁所有与你相连的束缚
command-disconnect_all_players-desc = 断开与服务器上连接的所有玩家
command-dismount-desc = 如果你在骑乘，请先下马，或者卸载骑在你身上的任何东西
command-dropall-desc = 把你所有的物品扔到地上
command-make_block-desc = 在你的位置生成一个具有颜色的方块
command-make_npc-desc =
    在你附近从配置中生成一个实体。
    使用 Tab 键获取示例或自动补全 。
command-dummy-desc = 生成一个训练假人
command-explosion-desc = 让地面爆炸
command-faction-desc = 向您的派系发送讯息
command-give_item-desc = 给自己一些物品，使用tab键获取示例或自动完成。
command-gizmos-desc = 管理小工具订阅。
command-gizmos_range-desc = 更改小工具订阅的范围。
command-goto-desc = 传送到某个位置
command-goto-rand = 传送到随机位置
command-group-desc = 向您的群组发送讯息
command-group_invite-desc = 邀请玩家加入群组
command-group_kick-desc = 从群组中移除玩家
command-group_leave-desc = 离开当前群组
command-group_promote-desc = 提升某玩家为群组领导者
command-health-desc = 设置您当前的生命值
command-into_npc-desc = 将自己转换为NPC，请谨慎使用!
command-join_faction-desc = 加入/离开指定的派系
command-jump-desc = 偏移您当前的位置
command-kick-desc = 踢出某个名称的玩家
command-kill-desc = 自杀
command-kill_npcs-desc = 杀死NPC
command-kit-desc = 将一组物品放入您的物品栏。
command-lantern-desc = 更改您的灯笼强度和颜色
command-light-desc = 生成具有光线的实体
command-lightning-desc = 在当前位置放出闪电
command-location-desc = 传送到某个地点
command-outcome-desc = 创建一个结果
command-permit_build-desc = 给予玩家在某范围内建造的权限
command-players-desc = 列出当前在线的玩家
command-portal-desc = 生成一个传送门
command-region-desc = 向您的区域内所有人发送讯息
command-reload_chunks-desc = 重新加载服务器上的区块
command-repair_equipment-desc = 修复所有以装备的物品
command-reset_recipes-desc = 重置您的配方书
command-respawn-desc = 传送到您的路径点
command-revoke_build-desc = 撤销玩家的建筑区域权限
command-revoke_build_all-desc = 撤销玩家所有区域的建筑权限
command-safezone-desc = 创建一个安全区域
command-say-desc = 向所有听的到的人发送讯息
command-scale-desc = 调整您的角色大小
command-server_physics-desc = 设置/取消账户的服务器物理授权
command-set_motd-desc = 设置服务器描述
command-tell-desc = 向另一个玩家发送讯息
command-tether-desc = 将另一个实体系在您身上
command-time-desc = 设置一天中的时间
command-time_scale-desc = 设置时间的缩放比例
# 当您不存在时，由 /disconnect_all 发出（？）
command-you-dont-exist = 您不存在，所以无法使用该命令
command-entity-has-no-client = 玩家没有客户端组件：{ $target }
command-inventory-cant-fit-item = 物品无法放入物品栏
command-kit-inventory-unavailable = 无法获取物品栏
command-unimplemented-spawn-special = 尚未实现特殊实体的生成
command-player-info-unavailable = 无法获取 { $target } 的玩家信息
command-no-dismount = 您没有骑乘或被骑乘
command-waypoint-error = 找不到您的路径点。
command-waypoint-result = 您当前的路径点位于 { $waypoint }；
command-parse-duration-error = 无法解析时长：{ $error }
command-client-has-no-socketaddr = 无法获取 { $target } 的套接字地址（通过 mpsc 连接）
command-dismounted = 已下马
command-destroyed-no-tethers = 您没有连接任何系绳
command-destroyed-tethers = 所有系绳已被摧毁！您现在自由了
command-death_effect-unknown = 未知的死亡效果 { $effect }
command-cannot-send-message-hidden = 处于隐身观察状态时无法发送消息。
command-spot-world_feature = 需要启用 `worldgen` 功能才能运行此指令。
command-spot-spot_not_found = 这个世界里没有找到那种类型的地点。
command-outcome-invalid_outcome = { $outcome } 不是有效的结果
command-outcome-expected_sprite_kind = 预期的 SpriteKind
command-outcome-expected_integer = 预期的整数
command-outcome-expected_frontent_specifier = 预期的前端规范
command-outcome-expected_skill_group_kind = 预期的有效 ron SkillGroupKind
command-outcome-expected_entity_arg = 预期的实体参数
command-outcome-expected_body_arg = 预期的身体参数
command-outcome-variant_expected = 预期的结果变体
command-whitelist-permission-denied = 没有权限移除用户：{ $username }
command-whitelist-unlisted = { $username } 不在白名单中
command-whitelist-removed = 已将 { $username } 从白名单中移除
command-whitelist-already-added = { $username } 已在白名单中！
command-whitelist-added = 已将 { $username } 添加到白名单
command-version-current = 服务器运行于 { $version }
command-unban-already-unbanned = { $player } 已经被解封
command-unban-successful = { $player } 已成功解封
command-unban-ip-successful = 通过用户 "{ $player }" 进行的 IP 封禁已成功解除（该用户仍处于封禁状态）
command-time_scale-changed = 设置时间比例为 { $scale }
command-time_scale-current = 当前时间比例为 { $scale }
command-sudo-no-permission-for-non-players = 您无权 sudo 非玩家
command-sudo-higher-role = 不能对角色比您高的玩家使用 sudo
command-site-not-found = 找不到场地
command-respawn-no-waypoint = 没有设置路径点
command-kick-higher-role = 不能踢出角色比您高的玩家
command-into_npc-warning = 希望您不是在滥用这个！
command-group_invite-invited-to-your-group = { $player } 已被邀请加入您的群组
command-group_invite-invited-to-group = 已邀请 { $player } 加入群组
command-group-join = 请先创建一个群组
command-faction-join = 请使用 /join_faction 加入派系
command-ban-already-added = { $player } 已在封禁名单上
command-ban-ip-queued = 已将 { $player } 加入常规封禁名单，并排队进行 IP 封禁，原因：{ $reason }
command-ban-ip-added = 已将 { $player } 加入常规封禁名单和 IP 封禁名单，原因：{ $reason }
command-ban-added = 已将 { $player } 添加到封禁名单，原因：{ $reason }
command-adminify-removed-role = 玩家 { $player } 的角色 { $role } 被移除
command-adminify-role-upgraded = 玩家 { $player } 的角色升级为 { $role }
command-adminify-role-downgraded = 玩家 { $player } 的角色降级为 { $role }
command-adminify-already-has-no-role = 玩家已经没有该角色！
command-adminify-already-has-role = 玩家已经拥有该角色！
command-adminify-cannot-find-player = 无法找到玩家实体！
command-adminify-reassign-to-above = 不能重新分配比您角色或更高的角色
command-adminify-assign-higher-than-own = 不能赋予比您自己永久角色更高的临时角色
command-experimental-terrain-persistence-disabled = 实验性地形持久化已禁用
command-server-no-experimental-terrain-persistence = 服务器编译时未启用地形持久化
command-reloaded-chunks = 重新加载了 { $reloaded } 区块
command-aura-spawn-new-entity = 生成了一个新光环
command-aura-spawn = 生成了一个附加在实体上的新光环
command-aura-invalid-buff-parameters = 光环的增益参数无效
command-transform-invalid-presence = 无法在当前状态下进行变形
command-tell-to-yourself = 您不能对自己使用 /tell
command-message-group-missing = 您正在使用群组聊天，但您不属于任何群组，请使用 /world 或 /region 来更改聊天模式
command-repaired-items = 修复了所有已装备的物品
command-repaired-inventory_items = 已修复所有物品
command-scale-set = 设置比例为 { $scale }
# 注意：不要翻译这些天气名称
command-weather-valid-values = 有效的值是 'clear', 'rain', 'wind' 和 'storm'
command-locations-list = 可用地点：{ $locations }
command-locations-empty = 目前没有任何地点
command-location-deleted = 已删除地点 '{ $location }'
command-location-created = 已创建地点 '{ $location }'
command-location-not-found = 地点 '{ $location }' 不存在
command-location-duplicate = 地点 '{ $location }' 已存在，请考虑先删除
command-location-invalid = 地点名称 '{ $location }' 无效，名称只能包含小写 ASCII 和底线
command-skillpreset-missing = 预设不存在：{ $preset }
command-skillpreset-broken = 技能预设损坏
command-skillpreset-load-error = 加载预设时出错
command-buff-body-unknown = 未知的身体规格：{ $spec }
command-buff-data = 增益参数 '{ $buff }' 需要附加数据
command-buff-unknown = 未知的增益效果：{ $buff }
command-battlemode-updated = 新的战斗模式：{ $battlemode }
command-battlemode-same = 尝试设置相同的战斗模式
command-battlemode-available-modes = 可用模式：pvp, pve
command-battlemode-cooldown = 冷却期间，请 { $cooldown } 秒后重试
command-battlemode-intown = 您需要在城镇中才能更改战斗模式！
command-disabled-by-settings = 服务器设置中禁用了该指令
command-unknown = 未知的指令
command-invalid-skill-group = { $group } 不是一个有效的技能组！
# 注意：此处不要翻译 "confirm"
command-disconnectall-confirm = 请再次运行该指令并添加第二个参数 "confirm" 以确认您确实想要断开所有玩家的服务器连接
command-explosion-power-too-low = 爆炸威力必须超过 { $power }
command-explosion-power-too-high = 爆炸威力不能超过 { $power }
command-lantern-adjusted-strength-color = 您调整了火焰强度和颜色
command-lantern-adjusted-strength = 您调整了火焰强度
command-lantern-unequiped = 请先装备灯笼
command-kit-not-enough-slots = 物品栏没有足够的空位
command-invalid-alignment = 无效的对齐方式：{ $alignment }
command-set_motd-message-not-set = 此本地化中没有设置 motd
command-set-waypoint-result = 路径点已设置！
command-set_motd-message-removed = 移除了服务器每日消息
command-set_motd-message-added = 服务器每日消息设置为 { $message }
command-set-build-mode-on-unpersistent = 打开了建筑模式，更改将在区块卸载时不再持久化
command-set-build-mode-on-persistent = 打开了建筑模式，实验性地形持久化已启用，服务器将尝试保存更改，但无法保证
command-set-build-mode-off = 关闭了建筑模式
command-no-buid-perms = 您无权建造
command-revoked-all-build = 已撤销所有建筑权限
command-revoke-build-all = 您的所有建筑权限已被撤销
command-revoke-build = 撤销了 '{ $area }' 的建筑权限
command-revoke-build-recv = 您在 '{ $area }' 的建筑权限已被撤销
command-permit-build-granted = 授予了在 '{ $area }' 建造的权限
command-permit-build-given = 您现在被允许在 '{ $area }' 建造
command-volume-created = 创建了一个体积
command-volume-size-incorrect = 大小必须在 1 到 127 之间
command-spawned-safezone = 生成了一个安全区域
command-spawned-campfire = 生成了一个营火
command-spawned-airship = 生成了一个飞空艇
command-spawned-dummy = 生成了一个训练假人
command-spawned-entity = 生成的实体 ID 为：{ $id }
command-chunk-out-of-bounds = 区块 { $x }, { $y } 不在地图范围内
command-chunk-not-loaded = 区块 { $x }, { $y } 未加载
command-rtsim-purge-perms = 您必须是真正的管理员（而不只是临时管理员）才能清除 rtsim 数据
command-time-unknown = 时间未知
command-time-current = 现在是 { $t }
command-time-invalid = { $t } 不是有效的时间
command-time-backwards = { $t } 在当前时间之前，时间不能倒退
command-time-parse-negative = { $n } 无效，不能为负数
command-time-parse-too-large = { $n } 无效，不能超过 16 位数字
command-invalid-sprite = 无效的精灵类型：{ $kind }
command-spawned-entities-config = 从配置 { $config } 生成了 { $n } 个实体
command-entity-load-failed = 加载实体配置失败：{ $config }
command-nof-entities-less-than = 实体数应小于 50
command-nof-entities-at-least = 实体数应至少为 1
command-invalid-block-kind = 无效的方块类型：{ $kind }
command-invalid-item = 无效的物品：{ $item }
command-give-inventory-success = 将 { $total } 件 { $item } 添加到物品栏
command-give-inventory-full =
    玩家物品栏已满，仅给予 { $given ->
        [1] 一件
       *[other] { $given } 件
    }，总共 { $total } 件物品
command-error-while-evaluating-request = 验证请求时遇到错误：{ $error }
command-error-write-settings =
    写入设置档至磁盘失败，但已成功写入内存
    错误（存储）：{ $error }
    成功（内存）：{ $message }
command-entity-dead = 实体 '{ $entity }' 已死亡！
command-no-sudo = 冒充他人是不礼貌的
command-uuid-username-unavailable = 无法为 UUID  { $uuid } 确定用户名
command-username-uuid-unavailable = 无法为用户名 { $username } 确定 UUID
command-player-uuid-not-found = 找不到 UUID 为 '{ $uuid }' 的玩家！
command-player-not-found = 找不到玩家 '{ $player }'！
command-area-not-found = 找不到名为 '{ $area }' 的区域
command-uid-unavailable = 无法获取 { $target } 的 UID
command-player-role-unavailable = 无法获取 { $target } 的管理员角色
command-position-unavailable = 无法获取 { $target } 的位置
command-no-permission = 您没有权限使用 '/{ $command_name }'
command-waypoint-desc = 设置当前位置为路径点
command-experimental-shaders-not-valid = 您必须指定一个有效的实验性着色器，不带任何参数使用此指令可以获取实验性着色器的列表。
command-experimental-shaders-not-a-shader = { $shader } 不是实验性着色器，带上任意参数使用此指令可以查看完整列表。
command-experimental-shaders-not-supported = 此游戏版本不支持 { $shader }
command-experimental-shaders-disabled = 已禁用 { $shader }
command-experimental-shaders-enabled = 已启用 { $shader }
command-experimental-shaders-not-found = 没有实验性着色器
command-experimental-shaders-list = { $shader-list }
command-shader-backend = 当前着色器后端：{ $shader-backend }
command-unmute-no-player-specified = 您必须指定一名要解除禁言的玩家
command-unmute-no-muted-player-found = 找不到名为 { $player } 的被禁言玩家
command-unmute-success = 已成功解除 { $player } 的禁言
command-unmute-cannot-unmute-self = 您不能解除自己的禁言
command-mute-no-player-specified = 您必须指定一名玩家
command-mute-already-muted = { $player } 已被禁言
command-mute-no-player-found = 找不到名为 { $player } 的玩家
command-mute-success = 已成功禁言 { $player }
command-mute-cannot-mute-self = 您不能禁言自己
command-invalid-command-message = 找不到名为 { $invalid-command } 的指令。
    你是不是想用下面这些？
    { $most-similar-command }
    { $commands-with-same-prefix }

    输入 /help 可以查看所有指令的列表。
command-preprocess-no-player-entity = 没有玩家实体
command-preprocess-not-valid-rider = 没有有效的骑手
command-preprocess-not-riding-valid-entity = 没有骑乘有效的实体
command-preprocess-not-valid-viewpoint-entity = 没有从有效的视角实体进行观察
command-preprocess-not-selected-valid-target = 没有选中有效的目标
command-preprocess-not-looking-at-valid-target = 没有看向有效的目标
command-preprocess-target-error = '@' 之后应为 { $expected_list }，实际为 { $target }
command-unmute-desc = 取消对使用静音指令静音的玩家
command-mute-desc = 静音某玩家的聊天消息
command-help-desc = 显示有关指令的信息
command-experimental_shader-desc = 切换实验性着色器
command-clear-desc = 清除聊天中的所有消息，影响所有聊天标签
command-world-desc = 向服务器上的所有人发送消息
players-list-header =
    { $count ->
        [1]
            { $count } 名玩家在线
            { $player_list }
       *[other]
            { $count } 名玩家在线
            { $player_list }
    }
command-naga-desc = 切换初始着色器处理中是否使用 naga（不会持久保存）
command-reset_tutorial-success = 教程状态已重置。
command-reset_tutorial-desc = 将游戏内教程重置为初始状态
command-wiki-success = wiki 指令执行成功
command-wiki-desc = 打开 wiki 或搜索某个主题
command-wiring-desc = 创建连接组件
command-whitelist-desc = 添加/移除白名单上的用户名
command-weather_zone-desc = 创建一个天气区域
command-version-desc = 显示服务器版本
command-unban-desc = 解除对给定用户名的封禁
command-unban-ip-desc = 只解除指定用户名的 IP 封禁。
command-rtsim_tp-desc = 传送到 rtsim npc
command-rtsim_purge-desc = 在下次启动时清除 rtsim 数据
command-rtsim_npc-desc = 按距离列出符合给定查询的 rtsim NPC（例如：模拟商人）
command-rtsim_info-desc = 显示 rtsim NPC 的信息
command-rtsim_chunk-desc = 显示当前区块的 rtsim 信息
command-tp-desc = 传送到另一个实体
command-make_sprite-desc = 在你的位置创建一个精灵。要定义精灵属性，请使用 RON 语法指定一个 StructureSprite。
command-make_volume-desc = 创建一个空间体积（实验性功能）
command-motd-desc = 查看服务器描述
command-mount-desc = 骑乘一个实体
command-object-desc = 生成一个物体
command-poise-desc = 设置你当前的姿态
command-remove_lights-desc = 移除所有由玩家生成的光源
command-set-waypoint-desc = 将你的航点设置为当前位置。
command-ship-desc = 生成一艘船
command-site-desc = 传送到一个地点
command-skill_point-desc = 为某个技能树分配技能点
command-skill_preset-desc = 赋予你的角色所需的技能。
command-spawn-desc = 生成一个测试实体
command-spot-desc = 查找并传送到最近的特定类型地点。
command-sudo-desc = 以另一个实体的身份运行命令
