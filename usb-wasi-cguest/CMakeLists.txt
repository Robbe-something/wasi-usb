cmake_minimum_required(VERSION 3.31)
project(usb_c_host C)

set(CMAKE_C_STANDARD 11)

include_directories(.)

add_executable(usb_c_host
        main.c)
