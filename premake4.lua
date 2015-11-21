solution "Blastoise"
    configurations { "Debug", "Release" }
    
    project "BltServer"
        kind "ConsoleApp"
        language "C++"
        files { "**.h", "**.cpp" }
        
        configuration "Debug"
            defines { "DEBUG" }
            flags { "Symbols" }
        
        configuration "Release"
            defines { "NDEBUG" }
            flags { "Optimize" }

    project "BltClient"
        kind "ConsoleApp"
        language "C++"

        files { "**.h", "**.cpp" }

        configuration "Debug"
            defines { "DEBUG" }
            flags { "Symbols" }

        configuration "Release"
            defines { "NDEBUG" }
            flags { "Optimize" }
