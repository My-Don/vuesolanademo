<script setup>
import { ref, watch, onMounted, computed } from "vue"
import { WalletMultiButton, useAnchorWallet } from "solana-wallets-vue"
import { 
  Connection, 
  PublicKey, 
  clusterApiUrl, 
  SystemProgram, 
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js"
import { Buffer } from "buffer"
import idl from "../../idl/idl.json"

// 全局设置Buffer
globalThis.Buffer = Buffer

defineProps({
  msg: {
    type: String,
    required: true,
  },
})

// 状态管理
const wallet = useAnchorWallet()
const isInitializing = ref(false)
const errorMessage = ref("")

// 将程序ID和PDA转换为响应式引用
const programID = ref(null)
const adminAddressPDA = ref(null)

// 初始化合约信息
onMounted(async () => {
  console.log("🔄 组件已挂载，初始化合约信息...")
  
  try {
    const contractAddress = "9Cmf94avwuwUo5zt8KphWJ68EoNjiCphrusdzMQE7Boi"
    programID.value = new PublicKey(contractAddress)
    console.log("✅ 程序ID创建成功:", programID.value?.toBase58())
    
    if (programID.value) {
      [adminAddressPDA.value] = await PublicKey.findProgramAddress(
        [Buffer.from("program_state")],
        programID.value
      )
      console.log("✅ PDA地址计算成功:", adminAddressPDA.value?.toBase58())
    }
  } catch (error) {
    console.error("初始化错误:", error)
  }
})

// 计算属性来监控钱包状态
const isWalletConnected = computed(() => {
  return !!(wallet.value && wallet.value.publicKey)
})

const walletAddress = computed(() => {
  if (wallet.value && wallet.value.publicKey) {
    return wallet.value.publicKey.toBase58()
  }
  return null
})

// 监听钱包状态变化
watch(() => wallet.value, (newVal) => {
  console.log("🔄 钱包状态变化:", newVal ? "已连接" : "未连接")
  
  if (newVal && newVal.publicKey) {
    console.log("✅ 钱包已连接:", newVal.publicKey.toBase58())
  }
}, { immediate: true })

// 连接rpc节点
const connection = new Connection(
  clusterApiUrl("devnet"),
  {
    commitment: "confirmed",
    confirmTransactionInitialTimeout: 60000,
    wsEndpoint: "wss://api.devnet.solana.com"
  }
)

// 生成Anchor方法的discriminator
const getMethodDiscriminator = async (methodName) => {
  try {
    // Anchor使用SHA256("global:" + methodName)的前8字节
    const encoder = new TextEncoder()
    const namespace = "global"
    const preimage = `${namespace}:${methodName}`
    
    // 使用Web Crypto API计算SHA256
    const msgUint8 = encoder.encode(preimage)
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgUint8)
    const hashArray = Array.from(new Uint8Array(hashBuffer))
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
    
    console.log(`🔑 ${methodName} discriminator: ${hashHex.slice(0, 16)}`)
    
    // 取前8字节（16个十六进制字符）
    return Buffer.from(hashHex.slice(0, 16), 'hex')
  } catch (error) {
    console.error("生成discriminator失败:", error)
    // 回退方案：使用已知的initialize discriminator
    // 这是Anchor中initialize方法的常见discriminator
    return Buffer.from([175, 175, 109, 31, 13, 152, 155, 237])
  }
}

// 方案1: 使用纯Web3.js API - 修复版（主要方案）// 方案1: 使用纯Web3.js API - 修复版（主要方案）
const initializeWithWeb3 = async () => {
  if (isInitializing.value || !isWalletConnected.value || !programID.value || !adminAddressPDA.value) {
    alert("请确保:\n1. 钱包已连接\n2. 合约信息已加载")
    return
  }
  
  isInitializing.value = true
  errorMessage.value = ""
  
  try {
    console.log("🚀 使用纯Web3.js API初始化合约...")
    console.log("📌 PDA地址:", adminAddressPDA.value.toBase58())
    console.log("👤 管理员地址:", wallet.value.publicKey.toBase58())
    
    // 步骤1: 获取正确的discriminator
    console.log("构建initialize指令...")
    // 使用我们计算出的正确discriminator
    const discriminator = Buffer.from("afaf6d1f0d989bed", "hex")
    console.log("🎯 Discriminator:", discriminator.toString('hex'))
    
    // 步骤2: 创建指令数据
    const adminPubkey = wallet.value.publicKey
    const adminBuffer = adminPubkey.toBuffer()
    
    // 组合指令数据: discriminator + admin公钥
    const instructionData = Buffer.concat([discriminator, adminBuffer])
    
    console.log("📊 指令数据长度:", instructionData.length, "字节")
    console.log("📊 指令数据hex:", instructionData.toString('hex'))
    
    // 步骤3: 创建交易指令
    // 根据IDL，账户顺序应该是：
    // 1. payer (signer, mutable)
    // 2. programState (mutable) 
    // 3. systemProgram (readonly)
    const keys = [
      { pubkey: wallet.value.publicKey, isSigner: true, isWritable: true }, // payer
      { pubkey: adminAddressPDA.value, isSigner: false, isWritable: true }, // programState
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false } // systemProgram
    ]
    
    console.log("📋 账户列表:")
    keys.forEach((key, i) => {
      console.log(`  ${i}: ${key.pubkey.toBase58()} 
        - signer: ${key.isSigner} 
        - writable: ${key.isWritable}`)
    })
    
    const instruction = new TransactionInstruction({
      keys,
      programId: programID.value,
      data: instructionData
    })
    
    // 步骤4: 计算所需租金
    console.log("💰 计算账户租金...")
    // 根据IDL: ProgramState {admin: pubkey, total_deposited: u64, bump: u8}
    // 总大小: 32 + 8 + 1 = 41 字节
    const stateSize = 41
    const rentExemption = await connection.getMinimumBalanceForRentExemption(stateSize)
    console.log(`租金要求: ${rentExemption} lamports (${stateSize}字节)`)
    
    // 检查钱包余额
    const walletBalance = await connection.getBalance(wallet.value.publicKey)
    console.log(`钱包余额: ${walletBalance} lamports (${(walletBalance / 1e9).toFixed(4)} SOL)`)
    
    if (walletBalance < rentExemption + 5000) {
      throw new Error(`余额不足！需要至少 ${(rentExemption + 5000) / 1e9} SOL 用于租金和手续费`)
    }
    
    // 步骤5: 创建并发送交易
    const transaction = new Transaction()
    transaction.add(instruction)
    transaction.feePayer = wallet.value.publicKey
    
    // 获取最新区块哈希
    console.log("⏳ 获取最新区块哈希...")
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed')
    transaction.recentBlockhash = blockhash
    
    console.log("📝 交易创建完成，准备签名...")
    
    // 步骤6: 签名并发送交易
    console.log("✍️ 签名交易...")
    const signedTransaction = await wallet.value.signTransaction(transaction)
    console.log("✅ 交易签名成功")
    
    // 先模拟交易
    console.log("🧪 模拟交易...")
    try {
      const simulation = await connection.simulateTransaction(signedTransaction)
      console.log("模拟结果:", simulation.value)
      
      if (simulation.value.err) {
        console.error("❌ 模拟失败:", simulation.value.err)
        throw new Error(`模拟失败: ${JSON.stringify(simulation.value.err)}`)
      }
      
      console.log("✅ 模拟成功，日志:")
      if (simulation.value.logs) {
        simulation.value.logs.forEach((log, i) => {
          console.log(`  [${i}] ${log}`)
        })
      }
    } catch (simError) {
      console.warn("⚠️ 模拟失败，但继续发送交易:", simError)
    }
    
    // 序列化交易
    const serializedTransaction = signedTransaction.serialize()
    console.log("📤 发送交易...")
    
    const signature = await connection.sendRawTransaction(serializedTransaction, {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 3
    })
    
    console.log("✅ 交易发送成功，签名:", signature)
    console.log("🔗 浏览器链接: https://explorer.solana.com/tx/" + signature + "?cluster=devnet")
    
    // 步骤7: 等待确认
    console.log("⏳ 等待交易确认...")
    const confirmation = await connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature
    }, 'confirmed')
    
    console.log("📋 交易确认状态:", confirmation.value)
    
    if (confirmation.value.err) {
      throw new Error(`交易失败: ${JSON.stringify(confirmation.value.err)}`)
    }
    
    // 验证PDA是否已创建
    console.log("🔍 验证PDA创建...")
    const accountInfo = await connection.getAccountInfo(adminAddressPDA.value, 'confirmed')
    if (!accountInfo) {
      throw new Error("PDA账户未创建，但交易已确认")
    }
    
    console.log("✅ PDA创建成功，数据长度:", accountInfo.data.length)
    
    const explorerUrl = `https://explorer.solana.com/tx/${signature}?cluster=devnet`
    alert(`🎉 合约初始化成功！
    
交易签名: ${signature}
管理员: ${wallet.value.publicKey.toBase58()}
PDA地址: ${adminAddressPDA.value.toBase58()}
数据大小: ${accountInfo.data.length} 字节
租金: ${(rentExemption / 1e9).toFixed(6)} SOL

查看交易: ${explorerUrl}`)
    
    window.open(explorerUrl, '_blank')
    
  } catch (error) {
    console.error("❌ 初始化失败:", error)
    errorMessage.value = error.message
    
    // 提供更详细的错误信息
    if (error.message.includes("0x65") || error.message.includes("InstructionFallbackNotFound")) {
      alert(`❌ 初始化失败：指令格式错误
      
错误代码: 0x65 (InstructionFallbackNotFound)
可能原因：
1. discriminator 不正确
2. 账户列表顺序错误
3. 程序ID不正确
4. 合约代码与IDL不匹配

已尝试：
✅ discriminator: afaf6d1f0d989bed
✅ 账户顺序: payer → programState → systemProgram
✅ 参数: admin公钥

请检查：
1. 合约中的initialize方法是否使用相同的discriminator
2. PDA种子是否为 "program_state"
3. 在Solana Explorer查看程序详情`)
    } else if (error.message.includes("custom program error")) {
      const errorCode = error.message.match(/0x[0-9a-fA-F]+/)?.[0] || '未知'
      alert(`❌ 合约执行错误
      
错误代码: ${errorCode}
      
常见错误代码：
0x0 - 成功
0x1 - 无效参数
0x65 - 指令未找到
0xBC - 账户已存在
      
建议：
1. 检查合约是否已部署到正确地址
2. 使用其他RPC节点重试
3. 查看合约日志获取更多信息`)
    } else {
      alert(`❌ 初始化失败:\n${error.message}`)
    }
    
  } finally {
    isInitializing.value = false
  }
}

// 方案3: 使用原生系统指令创建PDA（备用方案）
const initializeWithSystemProgram = async () => {
  if (isInitializing.value || !isWalletConnected.value || !programID.value || !adminAddressPDA.value) {
    alert("请确保:\n1. 钱包已连接\n2. 合约信息已加载")
    return
  }
  
  isInitializing.value = true
  errorMessage.value = ""
  
  try {
    console.log("🚀 使用系统程序直接创建账户...")
    
    // 计算PDA和bump
    const [pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("program_state")],
      programID.value
    )
    
    console.log("📌 PDA详细信息:")
    console.log("  地址:", pda.toBase58())
    console.log("  Bump:", bump)
    console.log("  种子: program_state")
    
    // 检查账户是否已存在
    const accountInfo = await connection.getAccountInfo(pda)
    if (accountInfo) {
      alert("账户已存在，无需初始化")
      return
    }
    
    // 计算租金
    const space = 41 // ProgramState大小
    const lamports = await connection.getMinimumBalanceForRentExemption(space)
    
    // 使用系统程序创建账户
    const createAccountIx = SystemProgram.createAccount({
      fromPubkey: wallet.value.publicKey,
      newAccountPubkey: pda,
      lamports,
      space,
      programId: programID.value,
    })
    
    // 然后调用initialize指令
    const discriminator = Buffer.from("afaf6d1f0d989bed", "hex")
    const instructionData = Buffer.concat([
      discriminator,
      wallet.value.publicKey.toBuffer()
    ])
    
    const initializeIx = new TransactionInstruction({
      keys: [
        { pubkey: wallet.value.publicKey, isSigner: true, isWritable: true },
        { pubkey: pda, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programID.value,
      data: instructionData,
    })
    
    // 创建交易
    const transaction = new Transaction()
    transaction.add(createAccountIx)
    transaction.add(initializeIx)
    transaction.feePayer = wallet.value.publicKey
    
    const { blockhash } = await connection.getLatestBlockhash('confirmed')
    transaction.recentBlockhash = blockhash
    
    // 签名并发送
    const signedTx = await wallet.value.signTransaction(transaction)
    const signature = await connection.sendRawTransaction(signedTx.serialize())
    
    console.log("✅ 交易发送成功:", signature)
    
    // 等待确认
    await connection.confirmTransaction(signature, 'confirmed')
    
    alert(`🎉 账户创建并初始化成功！
    
PDA地址: ${pda.toBase58()}
交易: ${signature}`)
    
  } catch (error) {
    console.error("❌ 备用方案失败:", error)
    alert(`备用方案失败: ${error.message}`)
  } finally {
    isInitializing.value = false
  }
}

// 检查程序详情
const checkProgramDetails = async () => {
  try {
    console.log("🔍 检查程序详情...")
    
    if (!programID.value) {
      alert("程序ID未设置")
      return
    }
    
    const accountInfo = await connection.getAccountInfo(programID.value, 'confirmed')
    
    if (!accountInfo) {
      alert("❌ 程序账户不存在或未部署")
      return
    }
    
    console.log("📊 程序账户信息:")
    console.log("  所有者:", accountInfo.owner.toBase58())
    console.log("  数据长度:", accountInfo.data.length, "字节")
    console.log("  Lamports:", accountInfo.lamports)
    console.log("  可执行:", accountInfo.executable)
    
    alert(`📊 程序详情：
地址: ${programID.value.toBase58()}
所有者: ${accountInfo.owner.toBase58()}
数据大小: ${accountInfo.data.length} 字节
余额: ${(accountInfo.lamports / 1e9).toFixed(6)} SOL
可执行: ${accountInfo.executable ? '✅ 是' : '❌ 否'}
    
状态: ${accountInfo.executable ? '✅ 已部署' : '❌ 未部署'}`)
    
  } catch (error) {
    console.error("检查失败:", error)
    alert(`检查失败: ${error.message}`)
  }
}

// 方案2: 使用@coral-xyz/anchor的方案（简化版）
const initializeWithSimpleAnchor = async () => {
  if (isInitializing.value || !isWalletConnected.value || !programID.value || !adminAddressPDA.value) {
    alert("请确保:\n1. 钱包已连接\n2. 合约信息已加载")
    return
  }
  
  isInitializing.value = true
  errorMessage.value = ""
  
  try {
    console.log("🚀 使用Anchor方案初始化合约...")
    
    // 动态导入@coral-xyz/anchor
    const anchorModule = await import('@coral-xyz/anchor')
    const anchor = anchorModule.default || anchorModule
    
    console.log("✅ Anchor模块加载成功")
    
    // 创建简单的Provider包装器
    const provider = {
      connection,
      wallet: {
        publicKey: wallet.value.publicKey,
        signTransaction: wallet.value.signTransaction,
        signAllTransactions: wallet.value.signAllTransactions
      },
      publicKey: wallet.value.publicKey,
      sendAndConfirm: async (transaction, signers, options) => {
        // 签名交易
        const signedTx = await wallet.value.signTransaction(transaction)
        
        // 如果有额外签名者，添加他们的签名
        if (signers && signers.length > 0) {
          signedTx.partialSign(...signers)
        }
        
        // 发送交易
        const rawTransaction = signedTx.serialize()
        const signature = await connection.sendRawTransaction(rawTransaction, {
          skipPreflight: options?.skipPreflight || false,
          preflightCommitment: options?.preflightCommitment || 'confirmed',
          maxRetries: 3
        })
        
        // 确认交易
        const latestBlockhash = await connection.getLatestBlockhash('confirmed')
        await connection.confirmTransaction({
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
          signature
        }, 'confirmed')
        
        return signature
      }
    }
    
    console.log("✅ Provider创建成功")
    
    // 创建Program实例
    const program = new anchor.Program(idl, programID.value, provider)
    console.log("✅ Program创建成功")
    
    // 调用initialize方法
    console.log("📝 调用initialize方法...")
    
    // 确保IDL中有initialize方法
    if (!program.methods.initialize) {
      throw new Error("IDL中没有找到initialize方法")
    }
    
    const txSignature = await program.methods
      .initialize(wallet.value.publicKey)
      .accounts({
        payer: wallet.value.publicKey,
        programState: adminAddressPDA.value,
        systemProgram: SystemProgram.programId
      })
      .rpc({
        skipPreflight: false,
        commitment: 'confirmed'
      })
    
    console.log("✅ 交易发送成功，签名:", txSignature)
    
    // 等待确认
    console.log("⏳ 等待交易确认...")
    await connection.confirmTransaction(txSignature, 'confirmed')
    
    const explorerUrl = `https://explorer.solana.com/tx/${txSignature}?cluster=devnet`
    alert(`🎉 合约初始化成功！
    
交易签名: ${txSignature}
管理员: ${wallet.value.publicKey.toBase58()}
PDA地址: ${adminAddressPDA.value.toBase58()}`)
    
    window.open(explorerUrl, '_blank')
    
  } catch (error) {
    console.error("❌ Anchor方案初始化失败:", error)
    errorMessage.value = error.message
    
    if (error.message.includes("_bn") || error.message.includes("undefined")) {
      console.log("⚠️ Anchor兼容性问题，建议使用Web3方案")
      alert(`⚠️ Anchor兼容性问题
      
建议使用"Web3方案"进行初始化
错误详情：${error.message}`)
    } else {
      alert(`❌ Anchor方案失败:\n${error.message}`)
    }
    
  } finally {
    isInitializing.value = false
  }
}

// 检查PDA状态（使用Web3.js直接查询）
const checkPDAState = async () => {
  if (!isWalletConnected.value || !programID.value || !adminAddressPDA.value) {
    alert("请确保:\n1. 钱包已连接\n2. 合约信息已加载")
    return
  }
  
  isInitializing.value = true
  errorMessage.value = ""
  
  try {
    console.log("🔍 检查PDA状态...")
    console.log("🔎 查询地址:", adminAddressPDA.value.toBase58())
    
    // 直接查询账户信息
    const accountInfo = await connection.getAccountInfo(adminAddressPDA.value, 'confirmed')
    
    if (!accountInfo) {
      console.log("ℹ️ PDA账户不存在")
      
      // 显示PDA详细信息
      const [pda, bump] = await PublicKey.findProgramAddress(
        [Buffer.from("program_state")],
        programID.value
      )
      
      alert(`📌 PDA详细信息：
地址: ${pda.toBase58()}
Bump: ${bump}
种子: "program_state"
程序ID: ${programID.value.toBase58()}
      
状态: ❌ 合约尚未初始化`)
      
    } else {
      console.log("✅ PDA账户存在，数据长度:", accountInfo.data.length)
      console.log("📊 账户数据hex:", accountInfo.data.slice(0, 64).toString('hex'))
      
      // 尝试解析账户数据
      if (accountInfo.data.length >= 41) {
        const adminPubkey = new PublicKey(accountInfo.data.slice(0, 32))
        const totalDeposited = accountInfo.data.readBigUInt64LE(32)
        const bump = accountInfo.data[40]
        
        alert(`📊 合约已初始化 ✅
管理员: ${adminPubkey.toBase58()}
总存款: ${totalDeposited.toString()} lamports
Bump: ${bump}
数据长度: ${accountInfo.data.length} 字节
        
地址: ${adminAddressPDA.value.toBase58()}`)
      } else {
        alert(`📊 PDA账户存在但数据格式异常
数据长度: ${accountInfo.data.length} 字节
地址: ${adminAddressPDA.value.toBase58()}
前16字节: ${accountInfo.data.slice(0, 16).toString('hex')}
      
可能原因：
1. 数据格式不匹配
2. 不是预期的程序状态账户`)
      }
    }
    
  } catch (error) {
    console.error("❌ 检查失败:", error)
    errorMessage.value = error.message
    alert(`检查失败:\n${error.message}`)
    
  } finally {
    isInitializing.value = false
  }
}

// 测试网络连接
const testConnection = async () => {
  try {
    const version = await connection.getVersion()
    console.log("✅ 网络连接正常，版本:", version)
    
    let balance = 0
    if (walletAddress.value) {
      balance = await connection.getBalance(new PublicKey(walletAddress.value), 'confirmed')
      console.log("💰 钱包余额:", (balance / 1e9).toFixed(4), "SOL")
    }
    
    alert(`🌐 网络连接正常 ✅
RPC节点: devnet
版本: ${version["solana-core"]}
${walletAddress.value ? `钱包余额: ${(balance / 1e9).toFixed(4)} SOL` : '钱包未连接'}`)
    
  } catch (error) {
    console.error("❌ 网络连接失败:", error)
    alert("❌ 网络连接失败，请检查网络设置")
  }
}

// 验证IDL结构
const validateIDLStructure = async () => {
  console.log("🔍 验证IDL结构...")
  
  if (!idl) {
    alert("❌ IDL文件未加载")
    return
  }
  
  try {
    // 检查initialize方法
    const initializeMethod = idl.instructions?.find(i => i.name === "initialize")
    
    if (!initializeMethod) {
      alert("❌ IDL中没有找到initialize方法")
      return
    }
    
    const details = `
IDL验证结果 ✅
- 版本: ${idl.version || "未知"}
- 名称: ${idl.name || "未知"}
- 指令数: ${idl.instructions?.length || 0}
- initialize方法: ✅ 存在

initialize方法详情:
账户: ${JSON.stringify(initializeMethod.accounts, null, 2)}
参数: ${JSON.stringify(initializeMethod.args, null, 2)}

建议检查:
1. 确保账户顺序正确
2. 确保参数类型匹配
3. PDA种子应为: "program_state"
    `
    
    console.log("📋 IDL详情:", details)
    
    // 同时计算并显示discriminator
    const discriminator = await getMethodDiscriminator("initialize")
    console.log("🎯 initialize discriminator:", discriminator.toString('hex'))
    
    alert("✅ IDL验证完成，请查看控制台获取详细信息")
    
  } catch (error) {
    console.error("验证IDL失败:", error)
    alert(`验证IDL失败:\n${error.message}`)
  }
}

// 测试钱包签名功能
const testWalletSign = async () => {
  if (!isWalletConnected.value) {
    alert("请先连接钱包")
    return
  }
  
  try {
    console.log("🧪 测试钱包签名功能...")
    
    // 创建一个简单的测试交易
    const transaction = new Transaction()
    transaction.feePayer = wallet.value.publicKey
    
    const { blockhash } = await connection.getLatestBlockhash('confirmed')
    transaction.recentBlockhash = blockhash
    
    // 添加一个无操作指令
    const instruction = SystemProgram.transfer({
      fromPubkey: wallet.value.publicKey,
      toPubkey: wallet.value.publicKey,
      lamports: 0
    })
    
    transaction.add(instruction)
    
    // 尝试签名
    const signed = await wallet.value.signTransaction(transaction)
    console.log("✅ 钱包签名测试成功")
    
    alert("✅ 钱包签名功能正常")
    
  } catch (error) {
    console.error("❌ 钱包签名测试失败:", error)
    alert(`钱包签名测试失败:\n${error.message}`)
  }
}

// 清理缓存重新开始
const resetAndRetry = async () => {
  console.log("🔄 重置状态重新开始...")
  
  isInitializing.value = true
  
  try {
    programID.value = null
    adminAddressPDA.value = null
    errorMessage.value = ""
    
    // 重新初始化
    const contractAddress = "9Cmf94avwuwUo5zt8KphWJ68EoNjiCphrusdzMQE7Boi"
    programID.value = new PublicKey(contractAddress)
    
    if (programID.value) {
      [adminAddressPDA.value] = await PublicKey.findProgramAddress(
        [Buffer.from("program_state")],
        programID.value
      )
    }
    
    console.log("✅ 重置完成")
    console.log("程序ID:", programID.value?.toBase58())
    console.log("PDA地址:", adminAddressPDA.value?.toBase58())
    
    alert("✅ 状态已重置，可以重新尝试")
    
  } catch (error) {
    console.error("重置失败:", error)
    alert(`重置失败:\n${error.message}`)
  } finally {
    isInitializing.value = false
  }
}

// 测试discriminator生成
const testDiscriminator = async () => {
  try {
    console.log("🧪 测试discriminator生成...")
    const disc = await getMethodDiscriminator("initialize")
    console.log("✅ discriminator:", disc.toString('hex'))
    alert(`Discriminator: ${disc.toString('hex')}`)
  } catch (error) {
    console.error("测试失败:", error)
    alert(`测试失败: ${error.message}`)
  }
}
</script>

<template>
  <div class="greetings">
    <div class="wallet-section">
      <wallet-multi-button></wallet-multi-button>
    </div>
    
    <!-- 状态显示 -->
    <div class="status-display">
      <div class="status-row">
        <div class="status-item">
          <span class="status-label">钱包状态:</span>
          <span class="status-value" :class="{ connected: isWalletConnected }">
            {{ isWalletConnected ? '✅ 已连接' : '❌ 未连接' }}
          </span>
        </div>
        
        <div class="status-item">
          <span class="status-label">程序ID:</span>
          <span class="status-value" :class="{ connected: programID }">
            {{ programID ? '✅ 已设置' : '❌ 未设置' }}
          </span>
        </div>
      </div>
      
      <div class="status-row">
        <div class="status-item">
          <span class="status-label">PDA地址:</span>
          <span class="status-value" :class="{ connected: adminAddressPDA }">
            {{ adminAddressPDA ? '✅ 已计算' : '❌ 未计算' }}
          </span>
        </div>
        
        <div class="status-item">
          <span class="status-label">操作状态:</span>
          <span class="status-value" :class="{ connected: !isInitializing }">
            {{ isInitializing ? '🔄 进行中' : '✅ 空闲' }}
          </span>
        </div>
      </div>
    </div>
    
    <!-- 主要操作按钮 -->
    <div class="main-actions">
      <h4>初始化合约方案</h4>
      <div class="action-buttons">
    <button 
      @click="initializeWithWeb3" 
      :disabled="!isWalletConnected || isInitializing || !programID || !adminAddressPDA"
      class="web3-btn"
    >
      {{ isInitializing ? "🔄 处理中..." : "⚡ Web3方案" }}
    </button>
    
    <button 
      @click="initializeWithSystemProgram" 
      :disabled="!isWalletConnected || isInitializing || !programID || !adminAddressPDA"
      class="system-btn"
    >
      {{ isInitializing ? "🔄 处理中..." : "🛠️ 系统方案" }}
    </button>
  </div>
  <p class="action-hint">推荐使用Web3方案，系统方案为备用方案</p>
    </div>
    
    <!-- 辅助工具按钮 -->
    <div class="tool-actions">
      <h4>辅助工具</h4>
      <div class="tool-buttons">
        <button @click="testConnection" class="tool-btn test">
          🌐 测试网络
        </button>
        
        <button @click="checkPDAState" class="tool-btn check">
          🔍 检查状态
        </button>
        
        <button @click="testWalletSign" class="tool-btn sign">
          ✍️ 测试签名
        </button>
        
        <button @click="validateIDLStructure" class="tool-btn validate">
          📄 验证IDL
        </button>
        
        <button @click="testDiscriminator" class="tool-btn test-disc">
          🎯 测试Disc
        </button>
        
        <button @click="resetAndRetry" class="tool-btn reset">
          🔁 重置状态
        </button>

        <button @click="checkProgramDetails" class="tool-btn program">
  📦      检查程序
      </button>

      </div>
    </div>
    
    <!-- 错误信息 -->
    <div v-if="errorMessage" class="error-message">
      <div class="error-header">
        <strong>❌ 错误信息</strong>
        <button @click="errorMessage = ''" class="close-btn">×</button>
      </div>
      <pre class="error-content">{{ errorMessage }}</pre>
    </div>
    
    <!-- 当前状态信息 -->
    <div class="current-info">
      <h4>当前状态信息</h4>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">钱包地址:</span>
          <span class="info-value" v-if="walletAddress">{{ walletAddress.slice(0, 8) }}...{{ walletAddress.slice(-8) }}</span>
          <span class="info-value" v-else>未连接</span>
        </div>
        
        <div class="info-item">
          <span class="info-label">程序ID:</span>
          <span class="info-value" v-if="programID">{{ programID.toBase58().slice(0, 8) }}...{{ programID.toBase58().slice(-8) }}</span>
          <span class="info-value" v-else>未设置</span>
        </div>
        
        <div class="info-item">
          <span class="info-label">PDA地址:</span>
          <span class="info-value" v-if="adminAddressPDA">{{ adminAddressPDA.toBase58().slice(0, 8) }}...{{ adminAddressPDA.toBase58().slice(-8) }}</span>
          <span class="info-value" v-else>未计算</span>
        </div>
      </div>
    </div>
    
    <!-- 操作说明 -->
    <div class="instructions">
      <h4>操作说明</h4>
      <ol>
        <li>连接钱包（点击上方钱包按钮）</li>
        <li>点击"测试网络"确保网络正常</li>
        <li>点击"检查状态"查看合约是否已初始化</li>
        <li>点击"验证IDL"确认IDL文件正确</li>
        <li><strong>推荐：点击"⚡ Web3方案"初始化合约</strong></li>
        <li>如果Web3方案失败，点击"🔗 Anchor方案"（备用）</li>
      </ol>
      <p class="warning-text">
        ⚠️ <strong>重要提示：</strong>请确保钱包中有足够的SOL支付交易费用和账户租金
      </p>
    </div>
    
    <h1 class="green">{{ msg }}</h1>
    <h3>
      You've successfully created a project with
      <a href="https://vite.dev/" target="_blank" rel="noopener">Vite</a> +
      <a href="https://vuejs.org/" target="_blank" rel="noopener">Vue 3</a>.
    </h3>
  </div>
</template>

<style scoped>
/* ... 保留原有的样式 ... */

.tool-btn.test-disc {
  background-color: #607d8b;
  color: white;
}

.warning-text {
  background-color: #fff3e0;
  padding: 12px;
  border-radius: 8px;
  border-left: 4px solid #ff9800;
  margin-top: 15px;
  font-size: 14px;
  color: #e65100;
}


.system-btn {
  flex: 1;
  max-width: 300px;
  padding: 20px;
  font-size: 18px;
  font-weight: 700;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.3s;
  min-height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #ff9800, #e65100);
  color: white;
}

.tool-btn.program {
  background-color: #4caf50;
  color: white;
}

</style>