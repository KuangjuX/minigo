from antlr4.ListTokenSource import ListTokenSource

from parser_.gen.GoParserVisitor import GoParserVisitor
from parser_.gen.GoLexer import GoLexer
from parser_.gen.GoParser import GoParser
from antlr4 import *
import llvmlite.ir as ir
from generator.types import EasyGoTypes
from generator.util import *
from generator.errors import *
from generator.symbol_table import SymbolTable, RedefinitionError


class EasyGoGenerator(GoParserVisitor):
    def __init__(self, error_listener=EasyGoErrorListener()):
        self.module = ir.Module()
        self.builder = ir.IRBuilder()
        self.symbol_table = SymbolTable()  # 符号表
        self.continue_block = None  # 当调用continue时应该跳转到的语句块
        self.break_block = None  # 当调用break时应该跳转到的语句块
        self.switch_context = None  # TODO
        self.current_base_type = None  # 当前上下文的基础数据类型
        self.is_global = True  # 当前是否处于全局环境中
        self.error_listener = error_listener  # 错误监听器
        self.global_context = ir.global_context
        self.struct_reflection = {}
        self.is_defining_struct = ''

    BASE_TYPE = 0
    ARRAY_TYPE = 1
    FUNCTION_TYPE = 2

    def visitFunctionDecl(self, ctx: GoParser.FunctionDeclContext):
        """
            functionDecl: FUNC IDENTIFIER (signature block?);
            return: void
        """
        self.is_global = False
        func_name = ctx.IDENTIFIER().getText()
        rt_type, params = self.visit(ctx.signature())
        arg_names = tuple(map(lambda x: x[0], params))
        param_types = tuple(map(lambda x: x[1], params))
        new_func_type = ir.FunctionType(rt_type, param_types)

        # 在当前作用域生成llvm函数
        if func_name in self.symbol_table:
            llvm_function = self.symbol_table[func_name]
            if llvm_function.function_type != new_func_type:
                raise SemanticError("Function {}'s definition different from its declaration".format(func_name), ctx)
        else:
            llvm_function = ir.Function(self.module, new_func_type, name=func_name)
            self.symbol_table[func_name] = llvm_function
        self.builder = ir.IRBuilder(llvm_function.append_basic_block(name="entry"))
        # 进入作用域,将参数添加到当前作用域
        self.symbol_table.enter_scope()
        try:
            for arg_name, llvm_arg in zip(arg_names, llvm_function.args):
                self.symbol_table[arg_name] = self.builder.alloca(llvm_arg.type)
                self.builder.store(llvm_arg, self.symbol_table[arg_name])
        except RedefinitionError as e:
            raise SemanticError(msg="Redefinition local variable {}".format(arg_name), ctx=ctx)

        self.continue_block = None
        self.break_block = None

        self.visit(ctx.block())

        if new_func_type.return_type == EasyGoTypes.void:
            self.builder.ret_void()
        self.symbol_table.exit_scope()
        self.is_global = True

    def visitAssignment(self, ctx: GoParser.AssignmentContext):
        """
            assignment: expressionList assign_op expressionList;
            return:
        """
        # 只解构第一个元素
        lhs, lhs_ptr = self.visit(ctx.expressionList(0))[0]
        rhs, _ = self.visit(ctx.expressionList(1))[0]

        target_type = lhs_ptr.type.pointee
        # if rhs.type != target_type:
        #     rhs = EasyGoTypes.cast_type(self.builder, target_type, rhs, ctx)
        op = self.visit(ctx.assign_op())
        if op == "=":
            self.builder.store(rhs, lhs_ptr)
        elif op == "+=":
            if EasyGoTypes.is_int(target_type):
                new_value = self.builder.add(lhs, rhs)
            elif EasyGoTypes.is_float(target_type):
                new_value = self.builder.fadd(lhs, rhs)
            self.builder.store(new_value, lhs_ptr)
        elif op == "-=":
            if EasyGoTypes.is_int(target_type):
                new_value = self.builder.sub(lhs, rhs)
            elif EasyGoTypes.is_float(target_type):
                new_value = self.builder.fsub(lhs, rhs)
            self.builder.store(new_value, lhs_ptr)

    def visitExpressionList(self, ctx: GoParser.ExpressionListContext):
        """
            expressionList: expression (COMMA expression)*;
            return: list[tuple(result,result_ptr/None)]
        """

        list = []
        for i in range(len(ctx.expression())):
            list.append(self.visit(ctx.expression(i)))
        return list

    def visitExpression(self, ctx: GoParser.ExpressionContext):
        """
            primaryExpr
                | unary_op = (
                  PLUS
                | MINUS
                ) expression .... and more in GoParser.g4

            return: (result,result_ptr/None)
        """
        if len(ctx.children) == 1:
            # 只有primaryExpr情况
            return self.visit(ctx.primaryExpr())
        elif len(ctx.children) == 3:
            # get left
            lhs, _ = self.visit(ctx.children[0])
            op = ctx.children[1].getText()
            # get right
            if match_rule(ctx.children[2], GoParser.RULE_expression):
                rhs, _ = self.visit(ctx.children[2])
            else:
                raise NotImplementedError("complex expression 1")

            ltype = lhs.type
            rtype = rhs.type
            if ltype != rtype:
                rhs = EasyGoTypes.cast_type(self.builder, ltype, rhs, ctx)

            # TODO: more operator to be implemented
            if EasyGoTypes.is_int(ltype):
                if op == '+':
                    return self.builder.add(lhs, rhs), _
                elif op == '-':
                    return self.builder.sub(lhs, rhs), _
                elif op == '*':
                    return self.builder.mul(lhs, rhs), _
                elif op == '/':
                    return self.builder.sdiv(lhs, rhs), _
                elif op in ['==', '!=', '>', '<', '>=', '<=']:
                    return self.builder.icmp_signed(cmpop=op, lhs=lhs, rhs=rhs), None
                else:
                    raise NotImplementedError("complex expression 2")
                pass
            else:
                if op == '+':
                    return self.builder.fadd(lhs, rhs), _
                elif op == '-':
                    return self.builder.fsub(lhs, rhs), _
                elif op == '*':
                    return self.builder.fmul(lhs, rhs), _
                elif op == '/':
                    return self.builder.fdiv(lhs, rhs), _
                elif op in ['==', '!=', '>', '<', '>=', '<=']:
                    return self.builder.fcmp_ordered(cmpop=op, lhs=lhs, rhs=rhs), None
                else:
                    raise NotImplementedError("complex expression 3")
                pass
        else:
            raise NotImplementedError("complex expression 4")
        pass

    def visitReturnStmt(self, ctx: GoParser.ReturnStmtContext):
        rt, _ = self.visit(ctx.expressionList())[0]
        self.builder.ret(rt)

    def visitPrimaryExpr(self, ctx: GoParser.PrimaryExprContext):
        """
        primaryExpr:
            operand ... and more
            return (result,result_ptr/None)
        """
        if len(ctx.children) == 1:
            return self.visit(ctx.operand())
        elif match_rule(ctx.children[0], GoParser.RULE_primaryExpr) and match_rule(ctx.children[1],
                                                                                   GoParser.RULE_arguments):
            # should call function here
            func, _ = self.visit(ctx.primaryExpr())
            arg_list = self.visit(ctx.arguments())
            if len(arg_list) > len(func.args):
                raise SyntaxError("Too much arguments! Don't support default value yet!")
            converted_args = [EasyGoTypes.cast_type(self.builder, value=arg[0], target_type=callee_arg.type, ctx=ctx)
                              for arg, callee_arg in zip(arg_list, func.args)]
            return self.builder.call(func, converted_args), None
            pass
        else:
            raise NotImplementedError("more primaryExpr")

    def visitArguments(self, ctx: GoParser.ArgumentsContext):
        return self.visit(ctx.expressionList())

    def visitOperand(self, ctx: GoParser.OperandContext):
        """
        operand: literal | operandName | L_PAREN expression R_PAREN;
        return (result,result_ptr/None)
        """
        if ctx.operandName():
            return self.visit(ctx.operandName())
        elif ctx.literal():
            return self.visit(ctx.literal())
        else:
            return self.visit(ctx.expression())

    def visitLiteral(self, ctx: GoParser.LiteralContext):
        """
            literal: basicLit | compositeLit | functionLit;
            return (result,result_ptr/None)
        """
        if ctx.basicLit():
            return self.visit(ctx.basicLit())
        else:
            raise NotImplementedError("more options of literal")

    def visitIfStmt(self, ctx: GoParser.IfStmtContext):
        cond_val, _ = self.visit(ctx.expression())
        converted_cond_val = EasyGoTypes.cast_type(self.builder, target_type=EasyGoTypes.bool, value=cond_val, ctx=ctx)
        self.symbol_table.enter_scope()
        blocks = ctx.block()
        if ctx.ifStmt():
            blocks.append(ctx.ifStmt())
        if len(blocks) == 2:  # 存在else分支
            with self.builder.if_else(converted_cond_val) as (then, otherwise):
                with then:
                    self.visit(blocks[0])
                with otherwise:
                    self.visit(blocks[1])
        else:  # 只有if分支
            with self.builder.if_then(converted_cond_val):
                self.visit(blocks[0])
        self.symbol_table.exit_scope()

    def visitForStmt(self, ctx: GoParser.ForStmtContext):
        self.symbol_table.enter_scope()
        name_prefix = self.builder.block.name
        cond_block = self.builder.append_basic_block(name_prefix + ".loop_cond")
        body_block = self.builder.append_basic_block(name_prefix + ".loop_body")
        end_block = self.builder.append_basic_block(name_prefix + ".loop_end")
        # condition
        cond_expression = ctx.expression()
        self.builder.branch(cond_block)
        self.builder.position_at_start(cond_block)
        if cond_expression:
            cond_val, _ = self.visit(cond_expression)
            converted_cond_val = EasyGoTypes.cast_type(self.builder, target_type=EasyGoTypes.bool, value=cond_val, ctx=ctx)
            self.builder.cbranch(converted_cond_val, body_block, end_block)
        else:
            self.builder.branch(body_block)
        # body
        self.builder.position_at_start(body_block)
        self.visit(ctx.block())
        self.builder.branch(cond_block)
        # end
        self.builder.position_at_start(end_block)
        self.symbol_table.exit_scope()

    def visitBasicLit(self, ctx: GoParser.BasicLitContext):
        """
            basicLit:
                NIL_LIT
                | integer
                | string_
                | FLOAT_LIT;
            return (result,result_ptr/None)
        """
        if ctx.integer():
            number = ctx.integer().getText()
            return EasyGoTypes.int(int(number)), None
        elif ctx.FLOAT_LIT():
            number = ctx.FLOAT_LIT().getText()
            return EasyGoTypes.float64(float(number)), None
        else:
            raise NotImplementedError("more options of BasicLit")

    def visitOperandName(self, ctx: GoParser.OperandNameContext):
        name = ctx.IDENTIFIER().getText()
        ptr = self.symbol_table[name]
        if type(ptr) in [ir.Argument, ir.Function]:
            var_val = ptr
            return var_val, None
        else:
            return self.builder.load(ptr), ptr

    def visitAssign_op(self, ctx: GoParser.Assign_opContext):
        return ctx.getText()

    def visitSignature(self, ctx: GoParser.SignatureContext):
        if ctx.result() is not None:
            rt_type = self.visit(ctx.result())
        else:
            rt_type = EasyGoTypes.void
        params = self.visit(ctx.parameters())
        return rt_type, params

    def visitParameters(self, ctx: GoParser.ParametersContext):
        param_list = []
        for i in range(len(ctx.parameterDecl())):
            param_list = param_list + self.visit(ctx.parameterDecl(i))
        return param_list

    def visitParameterDecl(self, ctx: GoParser.ParameterDeclContext):
        idn_list = self.visit(ctx.identifierList())
        type = self.visit(ctx.type_())
        rt_list = []
        for idn in idn_list:
            rt_list.append((idn, type))
        return rt_list

    def visitResult(self, ctx: GoParser.ResultContext):
        return self.visit(ctx.type_())

    def visitDeclaration(self, ctx: GoParser.DeclarationContext):
        """
                declaration: constDecl | typeDecl | varDecl;
                :return:void
                """
        self.visit(ctx.children[0])

    def visitVarDecl(self, ctx: GoParser.VarDeclContext):
        """
                varDecl: VAR (varSpec | L_PAREN (varSpec eos)* R_PAREN);
                return:void
        """
        self.visit(ctx.varSpec(0))

    def visitVarSpec(self, ctx: GoParser.VarSpecContext):
        """
        varSpec:
            identifierList (
            type_ (ASSIGN expressionList)?
            | ASSIGN expressionList
            );
        """
        idn_list = self.visit(ctx.identifierList())
        type = self.visit(ctx.type_())
        if ctx.expressionList():
            value, _ = self.visit(ctx.expressionList())[0]
            if value.type != type:
                value = EasyGoTypes.cast_type(self.builder, type, value, ctx)
        for var_name in idn_list:
            try:
                # if self.is_global:  # 如果是全局变量
                #     self.symbol_table[var_name] = ir.GlobalVariable(self.module, type, name=var_name)
                #     self.symbol_table[var_name].linkage = "internal"
                # else:  # 如果是局部变量
                self.symbol_table[var_name] = self.builder.alloca(type)
                # 如果有初始赋值就赋值
                if ctx.expressionList():
                    self.builder.store(value, self.symbol_table[var_name])
                pass
            except RedefinitionError as e:
                raise SemanticError(msg="redefinition variable {}".format(var_name), ctx=ctx)

    def visitIdentifierList(self, ctx: GoParser.IdentifierListContext):
        idn_list = list(map(lambda x: x.getText(), ctx.IDENTIFIER()))
        return idn_list

    def visitType_(self, ctx: GoParser.Type_Context):

        """
         type_: typeName | typeLit | L_PAREN type_ R_PAREN;
        """
        if len(ctx.children) == 3:
            # handle 3rd situation
            type = self.visit(ctx.type_())
        else:
            # handle the first one
            type = self.visit(ctx.typeName())

        if type in EasyGoTypes.str2type.keys():
            return EasyGoTypes.str2type[type]
        else:
            raise NotImplementedError("visitType_")

    def visitTypeName(self, ctx: GoParser.TypeNameContext):
        return str(ctx.children[0])

    def save(self, filename):
        """保存到文件"""
        with open(filename, "w") as f:
            f.write(repr(self.module))


def generate(input_filename, output_filename):
    """
    将Go代码文件转成IR代码文件
    :param input_filename: C代码文件
    :param output_filename: IR代码文件
    :return: 生成是否成功
    """
    lexer = GoLexer(FileStream(input_filename))
    tokensStream = CommonTokenStream(lexer)
    # parser = GoParser(stream)

    # tokens = lexer.getAllTokens()
    parser = GoParser(tokensStream)

    error_listener = EasyGoErrorListener()
    parser.removeErrorListeners()
    parser.addErrorListener(error_listener)

    tree = parser.sourceFile()
    print(tree.toStringTree(recog=parser))

    generator = EasyGoGenerator(error_listener)
    generator.visit(tree)
    generator.save(output_filename)

    if len(error_listener.errors) == 0:
        return True
    else:
        error_listener.print_errors()
        return False
