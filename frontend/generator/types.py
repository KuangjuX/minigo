import llvmlite.ir as ir
from generator.util import parse_escape
from generator.errors import SemanticError


class EasyGoTypes(object):
    int = ir.IntType(32)
    int16 = ir.IntType(16)
    bool = ir.IntType(1)
    float64 = ir.DoubleType()
    void = ir.VoidType()
    # 字符串到C变量类型映射表
    str2type = {
        "int": int,
        "int32": int,
        "int16": int16,
        "bool": bool,
        "float32": float64,
        "float64": float64,
        "void": void
    }
    # ASCII 转义表
    ascii_mapping = {
        '\\a': 7,
        '\\b': 8,
        '\\f': 12,
        '\\n': 10,
        '\\r': 13,
        '\\t': 9,
        '\\v': 11,
        '\\\\': 92,
        '\\?': 63,
        "\\'": 39,
        '\\"': 34,
        '\\0': 0,
    }

    @classmethod
    def get_const_from_str(cls, ctype, const_value, ctx):
        """
        从字符串获得常数类型
        :param ctype: 类型,接受char,float,double,short,int,ir.ArrayType
        :param const_value: 值，是一个字符串
        :return:
        """
        if type(const_value) is str:
            if ctype == cls.char:
                if len(const_value) == 3:  # 若const_value形如'3',
                    return cls.char(ord(str(const_value[1:-1])))  # 则将ASCII字符转成对应的整数存储
                elif len(const_value) == 1:  # 若const_value形如44
                    return cls.char(int(const_value))  # 则已经是整数了
                else:  # 若const_value是转移字符，例如'\n'
                    value = const_value[1:-1]
                    if value in cls.ascii_mapping:
                        return cls.char(cls.ascii_mapping[value])
                    else:
                        raise SemanticError(ctx=ctx, msg="Unknown char value: %s" % value)
            elif ctype in [cls.float, cls.double]:
                return ctype(float(const_value))
            elif ctype in [cls.short, cls.int]:
                return ctype(int(const_value))
            elif isinstance(ctype, ir.ArrayType) and ctype.element == cls.char:
                # string
                str_val = parse_escape(const_value[1:-1]) + '\0'
                return ir.Constant(ctype, bytearray(str_val, 'ascii'))
            else:
                # TODO
                raise SemanticError(msg="No known conversion: '%s' to '%s'" % (const_value, ctype))
        else:
            raise SyntaxError(ctx=ctx,
                              msg="get_const_from_str doesn't support const_value which is a " + str(type(const_value)))

    @classmethod
    def is_int(cls, type):
        """判断某个类型是否为整数类型"""
        return type in [cls.int, cls.int16]

    @classmethod
    def is_float(cls, type):
        """判断某个类型是否为浮点数类型"""
        return type in [cls.float64]

    @classmethod
    def cast_type(cls, builder, target_type, value, ctx):
        """
        强制类型转换
        :param builder:
        :param target_type:目标类型
        :param value:
        :return:转换后的数字
        """
        if value.type == target_type:  # 如果转换前后类型相同，
            return value  # 则不转换，直接返回

        if cls.is_int(value.type) or value.type == cls.bool:  # 从整数或者布尔值
            if cls.is_int(target_type):  # 转成整数
                if value.type.width < target_type.width:  # 扩展整数位数
                    return builder.sext(value, target_type)
                else:  # 减少整数位数
                    return builder.trunc(value, target_type)
            elif cls.is_float(target_type):  # 转成浮点数
                return builder.sitofp(value, target_type)
            elif target_type == cls.bool:
                return builder.icmp_unsigned('!=', value, cls.bool(0))
            elif type(target_type) == ir.PointerType:  # 转成指针
                return builder.inttoptr(value, target_type)

        elif cls.is_float(value.type):  # 从浮点数
            if cls.is_float(target_type):  # 转成浮点数
                if value.type == cls.float:  # 增加浮点数精度
                    return builder.fpext(value, cls.double)
                else:  # 降低浮点数精度
                    return builder.fptrunc(value, cls.float)
            elif cls.is_int(target_type):  # 转成整数
                return builder.fptosi(value, target_type)
        elif type(value.type) == ir.PointerType and type(target_type) == ir.IntType:
            # 指针转int
            return builder.ptrtoint(value, target_type)
        elif type(value.type) == ir.ArrayType and type(target_type) == ir.PointerType \
                and value.type.element == target_type.pointee:  # 数组类型转成指针类型
            zero = ir.Constant(cls.int, 0)
            tmp = builder.alloca(value.type)
            builder.store(value, tmp)
            return builder.gep(tmp, [zero, zero])
        elif isinstance(value.type, ir.ArrayType) and isinstance(target_type, ir.ArrayType) \
                and value.type.element == target_type.element:
            return builder.bitcast(value, target_type)
        elif isinstance(value.type, ir.PointerType) and isinstance(target_type, ir.PointerType):  # 指针之间的类型转换
            return builder.bitcast(value, target_type)
        raise SemanticError(ctx=ctx, msg="No known conversion from '%s' to '%s'" % (value.type, target_type))
