# 环境安装
> 目前在 mac（arm架构cpu）和 ubuntu 2204 测试可用
- 执行tools/setup_env.sh 脚本（需要sudo权限，但是**不要用root用户**）
- 安装python环境，需要`python >= 3.10`, 然后在arb_bot目录执行pip install -r requirements.txt
- 执行`solana-keygen new`生成一个钱包
- 执行`solana config get` 获取当前环境，生产的钱包私钥默认路径在`~/.config/solana/id.json`长度32的字节数组, 可以使用`python get_pubkey.py -f ~/.config/solana/id.json` 转换出对应公钥（**注意生成的私钥公钥不区分测试和主网，如果你在主网给这个地址充钱了，一定保留好私钥**）
- 修改anchor 测试使用的默认私钥地址，在Anchor.toml 中，修改`wallet = "~/.config/solana/id.json"`，这个路径是上一步 solana config get获取的wallet 路径

# 编译
在arb_bot 目录执行 `anchor build` 编译

# 测试
在arb_bot 目录执行 `anchor test` 编译&执行测试脚本（会本地拉起一个运行环境）
测试脚本在`tests/arb_touyi.ts`
测试脚本目前是调用合约里面的mytest指令（方法），传入了一个testUser账户，默认会直接使用solana 生成的默认路径钱包作为singer，所以这里没有传singer，只传了testUser，其他账户可用参考这种方式传递
